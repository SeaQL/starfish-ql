//! Graph query engine

mod executor;
mod worker;

use self::executor::Executor;
use crate::{
    entities::relation,
    schema::{format_edge_table_name, format_node_table_name}, lang::query::{QueryJson, QueryResultJson, QueryCommonConstraint, QueryVectorJson, QueryGraphJson, QueryVectorConstraintJson, QueryVectorConstraint, QueryConstraintSortByKeyJson},
};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbConn, DbErr, EntityTrait, FromQueryResult, Order, QueryFilter,
    Statement, JsonValue,
};
use sea_query::{Alias, Expr, SelectStatement};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{mem, collections::HashMap};

const BATCH_SIZE: usize = 300;
const DEBUG: bool = false;

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
/// A queried node
pub struct QueryResultNode {
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
/// A queried edge
pub struct QueryResultEdge {
    from_node: String,
    to_node: String,
}

#[derive(Debug)]
/// A helper struct to specify how to perform a graph query
pub struct QueryGraphParams {
    /// Which relation to consider for constructing the graph (unformatted)
    pub relation_name: Result<String,()>,
    /// Whether to reverse the direction when constructing the graph
    pub reverse_direction: bool,
    /// Specify the root nodes to be the union of ((the sets of nodes) each satisfying the conditions in one hash map)
    pub root_nodes_specifiers: Vec<HashMap<String, JsonValue>>,
    /// Recursion goes up to this level, 0 means no recursion at all.
    /// Recursion does not terminate early if this value is None.
    pub max_depth: Option<usize>,
    /// Sort each batch on this key in a Descending order (this value is an Unformatted column name)
    /// The order is random if this value is None.
    pub batch_sort_key: Option<String>,
    /// Include up to this number of nodes in each batch.
    /// All nodes are included in all batches if this value is None.
    pub max_batch_size: Option<usize>,
}

impl Default for QueryGraphParams {
    fn default() -> Self {
        Self {
            relation_name: Err(()),
            reverse_direction: false,
            root_nodes_specifiers: vec![],
            max_depth: Some(6),
            batch_sort_key: Some("in_conn".into()),
            max_batch_size: Some(6),
        }
    }
}

/// Query graph data
#[derive(Debug)]
pub struct Query;

impl Query {
    /// Query data from db
    pub async fn query(db: &DbConn, query_json: QueryJson) -> Result<QueryResultJson, DbErr> {
        match query_json {
            QueryJson::Vector(metadata) => Self::query_vector(db, metadata).await,
            QueryJson::Graph(metadata) => Self::query_graph(db, metadata).await,
        }
    }

    async fn query_vector(db: &DbConn, metadata: QueryVectorJson) -> Result<QueryResultJson, DbErr> {
        let mut stmt = sea_query::Query::select();
        
        stmt.column(Alias::new("name"))
            .from(Alias::new(&format_node_table_name(metadata.of)));

        for constraint in metadata.constraints {
            match constraint {
                QueryVectorConstraintJson::Common(constraint) => Self::handle_common_constraint(&mut stmt, constraint),
                QueryVectorConstraintJson::Exclusive(constraint) => Self::handle_vector_constraint(&mut stmt, constraint),
            }
        }

        let builder = db.get_database_backend();

        Ok(QueryResultJson::Vector(
            QueryResultNode::find_by_statement(builder.build(&stmt)).all(db).await?
        ))
    }

    async fn query_graph(db: &DbConn, metadata: QueryGraphJson) -> Result<QueryResultJson, DbErr> {

        Ok(QueryResultJson::Graph {
            nodes: vec![],
            edges: vec![],
        })
    }

    fn handle_common_constraint(stmt: &mut SelectStatement, constraint: QueryCommonConstraint) {
        match constraint {
            QueryCommonConstraint::SortBy(sort_by) => {
                let col_name = match sort_by.key {
                    QueryConstraintSortByKeyJson::Connectivity { of, r#type } => r#type.to_column_name(of)
                };
                stmt.order_by(Alias::new(&col_name), if sort_by.desc { Order::Desc } else { Order::Asc });
            },
            QueryCommonConstraint::Limit(limit) => {
                stmt.limit(limit);
            },
        }
    }

    fn handle_vector_constraint(stmt: &mut SelectStatement, constraint: QueryVectorConstraint) {
        match constraint {
            // Empty
        }
    }
}

/// Graph data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphData {
    /// Graph node data
    nodes: Vec<GraphNodeData>,
    /// Link data
    links: Vec<GraphLinkData>,
}

/// Graph node data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphNodeData {
    /// Name of node
    id: String,
    /// Weight
    weight: f64,
}

impl PartialEq for GraphNodeData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for GraphNodeData {}

impl std::hash::Hash for GraphNodeData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Tree data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreeData {
    /// Tree node data
    nodes: Vec<TreeNodeData>,
    /// Link data
    links: Vec<TreeLinkData>,
}

/// Tree node data
#[derive(Debug, Clone, Eq, Deserialize, Serialize)]
pub struct TreeNodeData {
    /// Name of node
    id: String,
    /// Node type
    r#type: TreeNodeType,
    /// Node depth inverse (the higher, the deeper in recursion this node was found)
    /// This field is not used to identify a tree node.
    depth_inv: i32,
}

impl PartialEq for TreeNodeData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.r#type == other.r#type
    }
}

impl std::hash::Hash for TreeNodeData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.r#type.hash(state);
    }
}

/// Denotes which side a node belongs to, relative to the **root** node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum TreeNodeType {
    /// Centered
    Root = 0,
    /// To the Left
    Dependency = 1,
    /// To the Right
    Dependent = 2,
}

/// Node weight option
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum NodeWeight {
    /// Simple (Immediatelly decay to 0)
    Simple = 0,
    /// Complex with weight decay factor 0.3
    FastDecay = 1,
    /// Complex with weight decay factor 0.5
    MediumDecay = 2,
    /// Complex with weight decay factor 0.7
    SlowDecay = 3,
    /// Compound (No decay)
    Compound = 4,
}

/// Graph link data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct GraphLinkData {
    /// Source node
    source: String,
    /// Target node
    target: String,
}

/// Tree link data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct TreeLinkData {
    /// Source node
    source: String,
    /// Target node
    target: String,
    /// Edge type
    r#type: TreeNodeType,
}

#[derive(Debug, Clone, FromQueryResult)]
struct Node {
    name: String,
    in_conn: f64,
}

#[derive(Debug, Clone, FromQueryResult)]
struct Link {
    from_node: String,
    to_node: String,
}

impl Query {
    /// Get graph
    pub async fn get_graph(
        db: &DbConn,
        relation_name: &str,
        top_n: i32,
        limit: i32,
        depth: i32,
        weight: NodeWeight,
    ) -> Result<GraphData, DbErr> {
        let relation = relation::Entity::find()
            .filter(relation::Column::Name.eq(relation_name))
            .one(db)
            .await?
            .ok_or_else(|| {
                DbErr::Custom(format!("Could not found relation '{}'", relation_name))
            })?;
        let to_entity = relation.to_entity.as_str();

        Executor::new(db)
            .get_graph(relation_name, to_entity, top_n, limit, depth, weight)
            .await
    }

    /// Get tree
    pub async fn get_tree(
        db: &DbConn,
        relation_name: &str,
        root_node: String,
        limit: i32,
        depth: i32,
        weight: NodeWeight,
    ) -> Result<TreeData, DbErr> {
        let relation = relation::Entity::find()
            .filter(relation::Column::Name.eq(relation_name))
            .one(db)
            .await?
            .ok_or_else(|| {
                DbErr::Custom(format!("Could not found relation '{}'", relation_name))
            })?;
        let to_entity = relation.to_entity.as_str();

        Executor::new(db)
            .get_tree(relation_name, to_entity, root_node, limit, depth, weight)
            .await
    }
}

#[allow(clippy::too_many_arguments)]
async fn traverse<N, L, SN, SL, CN, CL>(
    db: &DbConn,
    relation_name: &str,
    to_entity: &str,
    tree_node_type: TreeNodeType,
    weight: NodeWeight,
    select_nodes: SN,
    select_links: SL,
    convert_node: CN,
    convert_link: CL,
) -> Result<(Vec<N>, Vec<L>), DbErr>
where
    SN: FnOnce(&mut SelectStatement),
    SL: FnOnce(&SelectStatement) -> Vec<SelectStatement>,
    CN: Fn(Node) -> N,
    CL: Fn(Link) -> L,
{
    let builder = db.get_database_backend();
    let edge_table = &format_edge_table_name(relation_name);
    let node_table = &format_node_table_name(to_entity);
    let mut pending_nodes = Vec::new();
    let mut node_stmt = sea_query::Query::select();
    let in_conn = match weight {
        NodeWeight::Simple => format!("{}_in_conn", relation_name),
        NodeWeight::Compound => format!("{}_in_conn_compound", relation_name),
        NodeWeight::SlowDecay => format!("{}_in_conn_complex07", relation_name),
        NodeWeight::MediumDecay => format!("{}_in_conn_complex05", relation_name),
        NodeWeight::FastDecay => format!("{}_in_conn_complex03", relation_name),
    };
    node_stmt
        .column(Alias::new("name"))
        .expr_as(Expr::col(Alias::new(&in_conn)), Alias::new("in_conn"))
        .from(Alias::new(node_table));
    let mut edge_stmt = sea_query::Query::select();
    let join_col = match tree_node_type {
        TreeNodeType::Root => "",
        TreeNodeType::Dependency => "from_node",
        TreeNodeType::Dependent => "to_node",
    };
    edge_stmt
        .columns([Alias::new("from_node"), Alias::new("to_node")])
        .from(Alias::new(edge_table))
        .inner_join(
            Alias::new(node_table),
            Expr::tbl(Alias::new(node_table), Alias::new("name"))
                .equals(Alias::new(edge_table), Alias::new(join_col)),
        )
        .order_by((Alias::new(node_table), Alias::new(&in_conn)), Order::Desc);

    let edge_stmts = select_links(&edge_stmt);
    let links = if !edge_stmts.is_empty() {
        let edge_stmts: Vec<String> = edge_stmts
            .iter()
            .map(|stmt| builder.build(stmt).to_string())
            .collect();
        let union_select = format!("({})", edge_stmts.join(") UNION ("));
        let stmt = Statement::from_string(builder, union_select);
        let res_links = Link::find_by_statement(stmt).all(db).await?;
        pending_nodes = res_links
            .iter()
            .map(|edge| match tree_node_type {
                TreeNodeType::Root => unreachable!(),
                TreeNodeType::Dependency => edge.from_node.clone(),
                TreeNodeType::Dependent => edge.to_node.clone(),
            })
            .collect();
        res_links.into_iter().map(convert_link).collect()
    } else {
        vec![]
    };

    let node_stmt = if matches!(tree_node_type, TreeNodeType::Root) {
        select_nodes(&mut node_stmt);
        node_stmt
    } else {
        let target_nodes = mem::take(&mut pending_nodes);
        node_stmt.and_where(Expr::col(Alias::new("name")).is_in(target_nodes));
        node_stmt
    };

    let res_nodes = Node::find_by_statement(builder.build(&node_stmt))
        .all(db)
        .await?;
    let nodes = res_nodes.into_iter().map(convert_node).collect();

    Ok((nodes, links))
}

fn select_top_n_node(
    stmt: &mut SelectStatement,
    relation_name: &str,
    top_n: i32,
    weight: NodeWeight,
) {
    let in_conn = match weight {
        NodeWeight::Simple => "in_conn",
        NodeWeight::Compound => "in_conn_compound",
        NodeWeight::SlowDecay => "in_conn_complex07",
        NodeWeight::MediumDecay => "in_conn_complex05",
        NodeWeight::FastDecay => "in_conn_complex03",
    };
    stmt.order_by(
        Alias::new(&format!("{}_{}", relation_name, in_conn)),
        Order::Desc,
    )
    .limit(top_n as u64);
}

fn select_root_node(stmt: &mut SelectStatement, root_node: String) {
    stmt.and_where(Expr::col(Alias::new("name")).eq(root_node));
}

fn select_top_n_edge(
    stmt: &SelectStatement,
    limit: i32,
    nodes: Vec<String>,
    tree_node_type: TreeNodeType,
) -> Vec<SelectStatement> {
    if nodes.is_empty() {
        vec![]
    } else {
        nodes
            .into_iter()
            .map(|node| {
                let mut stmt = stmt.clone();
                let col = match tree_node_type {
                    TreeNodeType::Root => unreachable!(),
                    TreeNodeType::Dependency => "to_node",
                    TreeNodeType::Dependent => "from_node",
                };
                stmt.and_where(Expr::col(Alias::new(col)).eq(node))
                    .limit(limit as u64);
                stmt
            })
            .collect()
    }
}

fn into_graph_node(node: Node) -> GraphNodeData {
    GraphNodeData {
        id: node.name,
        weight: node.in_conn,
    }
}

fn into_graph_link(link: Link) -> GraphLinkData {
    GraphLinkData {
        source: link.from_node,
        target: link.to_node,
    }
}

fn into_tree_node(node: Node, r#type: TreeNodeType, depth_inv: i32) -> TreeNodeData {
    TreeNodeData {
        id: node.name,
        r#type,
        depth_inv,
    }
}

fn into_tree_link(link: Link, r#type: TreeNodeType) -> TreeLinkData {
    TreeLinkData {
        source: link.from_node,
        target: link.to_node,
        r#type,
    }
}
