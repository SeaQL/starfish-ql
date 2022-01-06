//! Graph query engine

use async_recursion::async_recursion;
use sea_orm::{Condition, ConnectionTrait, DbConn, DbErr, FromQueryResult, Order};
use sea_query::{Alias, Expr, SelectStatement};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, mem};

/// Query graph data
#[derive(Debug)]
pub struct Query;

/// Graph data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphData {
    /// Node data
    nodes: Vec<NodeData>,
    /// Link data
    links: Vec<LinkData>,
}

/// Node data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct NodeData {
    /// Name of node
    id: String,
    /// Weight
    weight: i32,
}

/// Link data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct LinkData {
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
struct Link {
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
impl Into<LinkData> for Link {
    fn into(self) -> LinkData {
        LinkData {
            source: self.from_node,
            target: self.to_node,
        }
    }
}

impl Query {
    /// Get graph
    pub async fn get_graph(db: &DbConn, top_n: i32, depth: i32) -> Result<GraphData, DbErr> {
        let mut pending_nodes = Vec::new();
        let mut nodes = HashSet::new();
        let mut links = HashSet::new();
        let mut node_stmt = sea_query::Query::select();
        node_stmt
            .columns([
                Alias::new("name"),
                Alias::new("in_conn"),
                Alias::new("out_conn"),
            ])
            .from(Alias::new("node_crate"));
        let mut edge_stmt = sea_query::Query::select();
        edge_stmt
            .columns([Alias::new("from_node"), Alias::new("to_node")])
            .from(Alias::new("edge_depends"));

        Self::traverse_graph(
            db,
            &mut pending_nodes,
            &mut nodes,
            &mut links,
            &node_stmt,
            &edge_stmt,
            top_n,
            depth,
            true,
        )
        .await?;

        Ok(GraphData {
            nodes: nodes.into_iter().collect(),
            links: links.into_iter().collect(),
        })
    }

    #[async_recursion]
    #[allow(clippy::too_many_arguments)]
    async fn traverse_graph(
        db: &DbConn,
        pending_nodes: &mut Vec<String>,
        nodes: &mut HashSet<NodeData>,
        links: &mut HashSet<LinkData>,
        node_stmt: &SelectStatement,
        edge_stmt: &SelectStatement,
        top_n: i32,
        depth: i32,
        first: bool,
    ) -> Result<(), DbErr> {
        if depth <= 0 || (!first && pending_nodes.is_empty()) {
            return Ok(());
        }

        // let pending_nodes = dbg!(pending_nodes);

        let builder = db.get_database_backend();
        let mut node_stmt = node_stmt.clone();
        if first {
            node_stmt
                .order_by_expr(
                    Expr::col(Alias::new("in_conn"))
                        .into_simple_expr()
                        .add(Expr::col(Alias::new("out_conn"))),
                    Order::Desc,
                )
                .limit(top_n as u64);
        } else {
            let target_nodes = mem::take(pending_nodes);
            node_stmt.and_where(Expr::col(Alias::new("name")).is_in(target_nodes));
        }

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
        links.extend(
            Link::find_by_statement(builder.build(&edge_stmt))
                .all(db)
                .await
                .map(|links| {
                    links
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
            links,
            &node_stmt,
            &edge_stmt,
            top_n,
            depth - 1,
            false,
        )
        .await?;

        Ok(())
    }
}
