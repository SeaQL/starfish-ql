//! Graph query engine

mod executor;
mod worker;

use self::executor::Executor;
use crate::{
    entities::relation,
    lang::query::{
        QueryCommonConstraint, QueryConstraintSortByKeyJson, QueryGraphConstraint,
        QueryGraphConstraintJson, QueryGraphConstraintLimitJson, QueryGraphJson, QueryJson,
        QueryResultJson, QueryVectorConstraint, QueryVectorConstraintJson, QueryVectorJson,
    },
    schema::{format_edge_table_name, format_node_table_name},
};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbConn, DbErr, EntityTrait, FromQueryResult, Order, QueryFilter,
    Statement,
};
use sea_query::{Alias, Cond, Expr, SelectStatement};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{collections::HashSet, mem};

const BATCH_SIZE: usize = 300;
const DEBUG: bool = false;

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
/// A queried node
pub struct QueryResultNode {
    /// Name of the node
    pub name: String,
    /// Associated weight (specified in query)
    pub weight: Option<f64>,
}

#[derive(Debug, Clone, FromQueryResult)]
/// A helper struct to temporarily store unique nodes
struct NodeName {
    name: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, FromQueryResult)]
/// A queried edge
pub struct QueryResultEdge {
    /// Name of the node in the from side
    pub from_node: String,
    /// Name of the node in the to side
    pub to_node: String,
}

impl QueryResultEdge {
    /// Convert self to an edge with flipped directions
    pub fn to_flipped(self) -> Self {
        Self {
            from_node: self.to_node,
            to_node: self.from_node,
        }
    }
}

#[derive(Debug)]
/// A helper struct to specify how to perform a graph query
pub struct QueryGraphParams {
    /// Which entity to consider for constructing the graph (unformatted)
    pub entity_name: Result<String, DbErr>,
    /// Which relation to consider for constructing the graph (unformatted)
    pub relation_name: Result<String, DbErr>,
    /// Whether to reverse the direction when constructing the graph
    pub reverse_direction: bool,
    /// Specify the root nodes to be the nodes with the supplied names
    /// The keys in the HashMaps must be Formatted.
    pub root_node_names: Vec<String>,
    /// Recursion goes up to this level, 0 means no recursion at all.
    /// Recursion does not terminate early if this value is None.
    pub max_depth: Option<u64>,
    /// Sort each batch on this key (this value is a Formatted column name).
    /// This key is also used as for filling the `weight` field of queried nodes, if supplied.
    /// The order is random if this value is None.
    pub batch_sort_key: Option<String>,
    /// Sort each batch in an ascending order if this value is true.
    pub batch_sort_asc: bool,
    /// Include up to this number of nodes in each batch.
    /// All nodes are included in all batches if this value is None.
    pub max_batch_size: Option<usize>,
    /// Include up to this number of nodes across the whole recursion.
    /// All nodes are included if this value is None.
    pub max_total_size: Option<usize>,
}

impl Default for QueryGraphParams {
    fn default() -> Self {
        Self {
            entity_name: Err(DbErr::Custom("Entity name is unspecified.".to_owned())),
            relation_name: Err(DbErr::Custom("Relation name is unspecified.".to_owned())),
            reverse_direction: false,
            root_node_names: vec![],
            max_depth: Some(6),
            batch_sort_key: None,
            batch_sort_asc: false,
            max_batch_size: Some(6),
            max_total_size: Some(10000),
        }
    }
}

impl QueryGraphParams {
    /// Construct params from metadata
    pub fn from_query_graph_metadata(metadata: QueryGraphJson) -> Self {
        let mut params = Self {
            entity_name: Ok(metadata.of),
            ..Default::default()
        };

        metadata
            .constraints
            .into_iter()
            .for_each(|constraint| match constraint {
                QueryGraphConstraintJson::Common(constraint) => {
                    params.handle_common_constraint(constraint)
                }
                QueryGraphConstraintJson::Exclusive(constraint) => {
                    params.handle_graph_constraint(constraint)
                }
            });

        params
    }

    fn handle_common_constraint(&mut self, constraint: QueryCommonConstraint) {
        match constraint {
            QueryCommonConstraint::SortBy(sort_by) => {
                self.batch_sort_key = match sort_by.key {
                    QueryConstraintSortByKeyJson::Connectivity { of, r#type } => {
                        Some(r#type.to_column_name(of))
                    }
                };
                self.batch_sort_asc = !sort_by.desc;
            }
            QueryCommonConstraint::Limit(limit) => self.max_total_size = Some(limit as usize),
        }
    }

    fn handle_graph_constraint(&mut self, constraint: QueryGraphConstraint) {
        match constraint {
            QueryGraphConstraint::Edge { of, traversal } => {
                self.relation_name = Ok(of);
                self.reverse_direction = traversal.reverse_direction;
            }
            QueryGraphConstraint::RootNodes(root_node_names) => {
                self.root_node_names = root_node_names;
            }
            QueryGraphConstraint::Limit(limit) => match limit {
                QueryGraphConstraintLimitJson::Depth(depth) => self.max_depth = depth,
                QueryGraphConstraintLimitJson::BatchSize(batch_size) => {
                    self.max_batch_size = batch_size
                }
            },
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

    async fn query_vector(
        db: &DbConn,
        metadata: QueryVectorJson,
    ) -> Result<QueryResultJson, DbErr> {
        let mut stmt = sea_query::Query::select();

        stmt.column(Alias::new("name"))
            .expr_as(Expr::value(Option::<f64>::None), Alias::new("weight"))
            .from(Alias::new(&format_node_table_name(metadata.of)));

        for constraint in metadata.constraints {
            match constraint {
                QueryVectorConstraintJson::Common(constraint) => {
                    Self::handle_common_constraint(&mut stmt, constraint)
                }
                QueryVectorConstraintJson::Exclusive(constraint) => {
                    Self::handle_vector_constraint(&mut stmt, constraint)
                }
            }
        }

        let builder = db.get_database_backend();

        Ok(QueryResultJson::Vector(
            QueryResultNode::find_by_statement(builder.build(&stmt))
                .all(db)
                .await?,
        ))
    }

    fn handle_common_constraint(stmt: &mut SelectStatement, constraint: QueryCommonConstraint) {
        match constraint {
            QueryCommonConstraint::SortBy(sort_by) => {
                let col_name = match sort_by.key {
                    QueryConstraintSortByKeyJson::Connectivity { of, r#type } => {
                        r#type.to_column_name(of)
                    }
                };
                stmt.expr_as(Expr::col(Alias::new(&col_name)), Alias::new("weight"))
                    .order_by(
                        Alias::new(&col_name),
                        if sort_by.desc {
                            Order::Desc
                        } else {
                            Order::Asc
                        },
                    );
            }
            QueryCommonConstraint::Limit(limit) => {
                stmt.limit(limit);
            }
        }
    }

    fn handle_vector_constraint(stmt: &mut SelectStatement, constraint: QueryVectorConstraint) {
        match constraint {
            // Empty
        }
    }

    async fn query_graph(db: &DbConn, metadata: QueryGraphJson) -> Result<QueryResultJson, DbErr> {
        let params = QueryGraphParams::from_query_graph_metadata(metadata);

        Self::traverse_with_params(db, params).await
    }

    async fn traverse_with_params(
        db: &DbConn,
        params: QueryGraphParams,
    ) -> Result<QueryResultJson, DbErr> {
        let builder = db.get_database_backend();
        let edge_table = &format_edge_table_name(params.relation_name?);
        let node_table = &format_node_table_name(params.entity_name?);

        // Start with root nodes
        let mut pending_nodes: Vec<String> = {
            let root_node_stmt = sea_query::Query::select()
                .column(Alias::new("name"))
                .from(Alias::new(node_table))
                .cond_where(
                    params
                        .root_node_names
                        .into_iter()
                        .fold(Cond::any(), |cond, name| {
                            cond.add(Expr::col(Alias::new("name")).eq(name))
                        }),
                )
                .to_owned();

            NodeName::find_by_statement(builder.build(&root_node_stmt))
                .all(db)
                .await?
                .into_iter()
                .map(|node| node.name)
                .collect()
        };

        let mut result_nodes: HashSet<String> = HashSet::from_iter(pending_nodes.iter().cloned());
        let mut result_edges: HashSet<QueryResultEdge> = HashSet::new();

        // Normal direction: Join on "from" -> finding "to"'s
        // Reverse: Join on "to" -> finding "from"'s
        let join_col = if !params.reverse_direction {
            "from_node"
        } else {
            "to_node"
        };

        let mut depth = 0;
        while params.max_depth.is_none() || depth < params.max_depth.unwrap() {
            // Fetch target edges from pending_nodes
            let target_edges = {
                let target_edge_stmt = sea_query::Query::select()
                    .columns([Alias::new("from_node"), Alias::new("to_node")])
                    .from(Alias::new(edge_table))
                    .inner_join(
                        Alias::new(node_table),
                        Expr::tbl(Alias::new(node_table), Alias::new("name"))
                            .equals(Alias::new(edge_table), Alias::new(join_col)),
                    )
                    .cond_where(
                        pending_nodes
                            .into_iter()
                            .fold(Cond::any(), |cond, node_name| {
                                cond.add(Expr::col(Alias::new(join_col)).eq(node_name))
                            }),
                    )
                    .to_owned();

                QueryResultEdge::find_by_statement(builder.build(&target_edge_stmt))
                    .all(db)
                    .await?
            };

            let mut total_nodes_full = false;

            pending_nodes = target_edges
                .into_iter()
                .filter_map(|edge| {
                    let target_node_name = if !params.reverse_direction {
                        edge.to_node.clone()
                    } else {
                        edge.from_node.clone()
                    };

                    if result_edges.insert(edge) && !result_nodes.contains(&target_node_name) {
                        if let Some(max_total_size) = params.max_total_size {
                            if result_nodes.len() >= max_total_size {
                                total_nodes_full = true;
                            }
                        }
                        Some(target_node_name)
                    } else {
                        None
                    }
                })
                .collect();

            // Sort by specified key if appropriate
            if let Some(order_by_key) = &params.batch_sort_key {
                pending_nodes = {
                    let stmt = sea_query::Query::select()
                        .column(Alias::new("name"))
                        .from(Alias::new(node_table))
                        .cond_where(pending_nodes.into_iter().fold(Cond::any(), |cond, name| {
                            cond.add(Expr::col(Alias::new("name")).eq(name))
                        }))
                        .order_by(
                            Alias::new(order_by_key),
                            if params.batch_sort_asc {
                                Order::Asc
                            } else {
                                Order::Desc
                            },
                        )
                        .to_owned();

                    NodeName::find_by_statement(builder.build(&stmt))
                        .all(db)
                        .await?
                        .into_iter()
                        .map(|node| node.name)
                        .collect()
                };
            }

            if let Some(max_batch_size) = params.max_batch_size {
                if max_batch_size < pending_nodes.len() {
                    pending_nodes = pending_nodes[0..max_batch_size].to_vec();
                }
            }

            result_nodes.extend(pending_nodes.iter().cloned());

            if pending_nodes.is_empty() || total_nodes_full {
                break;
            }

            depth += 1;
        }

        // Make sure all edges in result_edges use only nodes in result_nodes
        let edges: Vec<QueryResultEdge> = {
            let iter = result_edges.into_iter().filter(|edge| {
                result_nodes.contains(&edge.from_node) && result_nodes.contains(&edge.to_node)
            });

            if params.reverse_direction {
                iter.map(|edge| edge.to_flipped()).collect()
            } else {
                iter.collect()
            }
        };

        // Fetch the weights if needed
        let nodes: Vec<QueryResultNode> = if let Some(weight_key) = params.batch_sort_key {
            let stmt = sea_query::Query::select()
                .column(Alias::new("name"))
                .expr_as(Expr::col(Alias::new(&weight_key)), Alias::new("weight"))
                .from(Alias::new(node_table))
                .cond_where(result_nodes.into_iter().fold(Cond::any(), |cond, name| {
                    cond.add(Expr::col(Alias::new("name")).eq(name))
                }))
                .to_owned();

            QueryResultNode::find_by_statement(builder.build(&stmt))
                .all(db)
                .await?
        } else {
            result_nodes
                .into_iter()
                .map(|name| QueryResultNode { name, weight: None })
                .collect()
        };

        Ok(QueryResultJson::Graph { nodes, edges })
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
