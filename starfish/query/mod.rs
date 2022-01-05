//! Graph query engine

use async_recursion::async_recursion;
use sea_orm::{Condition, ConnectionTrait, DbConn, DbErr, FromQueryResult};
use sea_query::{Alias, Expr, SelectStatement};
use serde::{Deserialize, Serialize};
use std::mem;

/// Query graph data
#[derive(Debug)]
pub struct Query;

/// Graph data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphData {
    /// Node data
    nodes: Vec<NodeData>,
    /// Edge data
    links: Vec<EdgeData>,
}

/// Node data
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeData {
    /// Name of node
    id: String,
    /// Weight
    weight: i32,
}

/// Edge data
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Deserialize, Serialize)]
pub struct EdgeData {
    /// Source node
    source: String,
    /// Target node
    target: String,
}

#[derive(Debug, FromQueryResult)]
struct Node {
    name: String,
    in_conn: i32,
    out_conn: i32,
}

#[derive(Debug, FromQueryResult)]
struct Edge {
    from_node: String,
    to_node: String,
}

#[allow(clippy::from_over_into)]
impl Into<NodeData> for Node {
    fn into(self) -> NodeData {
        NodeData {
            id: self.name,
            weight: self.in_conn + self.out_conn,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<EdgeData> for Edge {
    fn into(self) -> EdgeData {
        EdgeData {
            source: self.from_node,
            target: self.to_node,
        }
    }
}

impl Query {
    /// Get graph
    pub async fn get_graph(
        db: &DbConn,
        root_min_in_conn: i32,
        root_min_out_conn: i32,
        depth: i32,
    ) -> Result<GraphData, DbErr> {
        let mut pending_nodes = Vec::new();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_stmt = sea_query::Query::select();
        node_stmt.from(Alias::new("node_crate")).columns([
            Alias::new("name"),
            Alias::new("in_conn"),
            Alias::new("out_conn"),
        ]);
        let mut edge_stmt = sea_query::Query::select();
        edge_stmt
            .from(Alias::new("edge_depends"))
            .columns([Alias::new("from_node"), Alias::new("to_node")]);

        Self::traverse_graph(
            db,
            &mut pending_nodes,
            &mut nodes,
            &mut edges,
            &node_stmt,
            &edge_stmt,
            root_min_in_conn,
            root_min_out_conn,
            depth,
            true,
        )
        .await?;

        nodes.sort();
        edges.sort();

        nodes.dedup();
        edges.dedup();

        Ok(GraphData {
            nodes,
            links: edges,
        })
    }

    #[async_recursion]
    #[allow(clippy::too_many_arguments)]
    async fn traverse_graph(
        db: &DbConn,
        pending_nodes: &mut Vec<String>,
        nodes: &mut Vec<NodeData>,
        edges: &mut Vec<EdgeData>,
        node_stmt: &SelectStatement,
        edge_stmt: &SelectStatement,
        root_min_in_conn: i32,
        root_min_out_conn: i32,
        depth: i32,
        first: bool,
    ) -> Result<(), DbErr> {
        if depth <= 0 || (!first && pending_nodes.is_empty()) {
            return Ok(());
        }

        // let pending_nodes = dbg!(pending_nodes);

        let builder = db.get_database_backend();
        let mut node_stmt = node_stmt.clone();
        node_stmt.cond_where(
            Condition::any()
                .add(Expr::col(Alias::new("in_conn")).gte(root_min_in_conn))
                .add(Expr::col(Alias::new("out_conn")).gte(root_min_out_conn)),
        );

        // dbg!(builder.build(&node_stmt));

        nodes.extend(
            Node::find_by_statement(builder.build(&node_stmt))
                .all(db)
                .await
                .map(|nodes| {
                    nodes
                        .into_iter()
                        .map(|node| {
                            pending_nodes.push(node.name.clone());
                            node.into()
                        })
                        .collect::<Vec<_>>()
                })?
                .into_iter(),
        );

        let mut edge_stmt = edge_stmt.clone();
        let from_nodes = mem::take(pending_nodes);
        // let from_nodes = dbg!(from_nodes);
        edge_stmt.and_where(Expr::col(Alias::new("from_node")).is_in(from_nodes));

        // dbg!(builder.build(&edge_stmt));

        let mut pending_nodes = Vec::new();
        edges.extend(
            Edge::find_by_statement(builder.build(&edge_stmt))
                .all(db)
                .await
                .map(|edges| {
                    edges
                        .into_iter()
                        .map(|edge| {
                            pending_nodes.push(edge.to_node.clone());
                            edge.into()
                        })
                        .collect::<Vec<_>>()
                })?
                .into_iter(),
        );

        // let mut pending_nodes = dbg!(pending_nodes);

        Self::traverse_graph(
            db,
            &mut pending_nodes,
            nodes,
            edges,
            &node_stmt,
            &edge_stmt,
            root_min_in_conn,
            root_min_out_conn,
            depth - 1,
            false,
        )
        .await?;

        Ok(())
    }
}
