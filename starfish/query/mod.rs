//! Graph query engine

use async_recursion::async_recursion;
use sea_orm::{ConnectionTrait, DbConn, DbErr, FromQueryResult, Order, Statement, Value, Values};
use sea_query::{Alias, Expr, SelectStatement};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, mem};

/// Query graph data
#[derive(Debug)]
pub struct Query;

/// Graph data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphData {
    /// Graph node data
    nodes: Vec<GraphNodeData>,
    /// Link data
    links: Vec<LinkData>,
}

/// Graph node data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct GraphNodeData {
    /// Name of node
    id: String,
    /// Weight
    weight: i32,
}

/// Tree data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreeData {
    /// Tree node data
    nodes: Vec<TreeNodeData>,
    /// Link data
    links: Vec<LinkData>,
}

/// Tree node data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct TreeNodeData {
    /// Name of node
    id: String,
    /// Node type
    r#type: TreeNodeType,
}

/// Denotes which side a node belongs to, relative to the **root** node
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum TreeNodeType {
    /// Centered
    Root = 0,
    /// To the Left
    Dependency = 1,
    /// To the Right
    Dependent = 2,
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
}

#[derive(Debug, FromQueryResult)]
struct Link {
    from_node: String,
    to_node: String,
}

#[allow(clippy::from_over_into)]
impl Into<GraphNodeData> for Node {
    fn into(self) -> GraphNodeData {
        GraphNodeData {
            id: self.name,
            weight: self.in_conn,
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
    pub async fn get_graph(
        db: &DbConn,
        top_n: i32,
        limit: i32,
        depth: i32,
    ) -> Result<GraphData, DbErr> {
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
            limit,
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
        nodes: &mut HashSet<GraphNodeData>,
        links: &mut HashSet<LinkData>,
        node_stmt: &SelectStatement,
        edge_stmt: &SelectStatement,
        top_n: i32,
        limit: i32,
        depth: i32,
        first: bool,
    ) -> Result<(), DbErr> {
        let builder = db.get_database_backend();

        if !pending_nodes.is_empty() {
            let mut stmts = Vec::new();
            for node in pending_nodes.iter() {
                let mut stmt = edge_stmt.clone();
                stmt.and_where(Expr::col(Alias::new("to_node")).eq(node.as_str()))
                    .limit(limit as u64);
                let stmt = builder.build(&stmt).to_string();
                stmts.push(stmt);
            }
            let stmt = Statement::from_string(builder, format!("({})", stmts.join(") UNION (")));
            pending_nodes.clear();

            links.extend(
                Link::find_by_statement(stmt)
                    .all(db)
                    .await
                    .map(|links| {
                        links
                            .into_iter()
                            .map(|edge| {
                                pending_nodes.push(edge.from_node.clone());
                                edge.into()
                            })
                            .collect::<Vec<_>>()
                    })?
                    .into_iter(),
            );
        }

        let mut stmt = node_stmt.clone();
        if first {
            stmt.order_by(Alias::new("in_conn"), Order::Desc)
                .limit(top_n as u64);
        } else {
            stmt.and_where(Expr::col(Alias::new("name")).is_in(pending_nodes.clone()));
        }

        nodes.extend(
            Node::find_by_statement(builder.build(&stmt))
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

        if depth > 0 && !pending_nodes.is_empty() {
            Self::traverse_graph(
                db,
                pending_nodes,
                nodes,
                links,
                node_stmt,
                edge_stmt,
                top_n,
                limit,
                depth - 1,
                false,
            )
            .await?;
        }

        Ok(())
    }

    // /// Get tree
    // pub async fn get_tree(db: &DbConn, root_node: String, depth: i32) -> Result<TreeData, DbErr> {
    //     let mut pending_lib_nodes = Vec::new();
    //     let mut pending_app_nodes = Vec::new();
    //     let mut nodes = Vec::new();
    //     let mut links = Vec::new();
    //     let mut node_stmt = sea_query::Query::select();
    //     node_stmt
    //         .columns([
    //             Alias::new("name"),
    //             Alias::new("in_conn"),
    //             Alias::new("out_conn"),
    //         ])
    //         .from(Alias::new("node_crate"));
    //     let mut edge_stmt = sea_query::Query::select();
    //     edge_stmt
    //         .columns([Alias::new("from_node"), Alias::new("to_node")])
    //         .from(Alias::new("edge_depends"));

    //     Self::traverse_tree(
    //         db,
    //         &mut pending_lib_nodes,
    //         &mut nodes,
    //         &mut links,
    //         &node_stmt,
    //         &edge_stmt,
    //         &TreeNodeType::Dependency,
    //         depth,
    //     )
    //     .await?;

    //     Self::traverse_tree(
    //         db,
    //         &mut pending_app_nodes,
    //         &mut nodes,
    //         &mut links,
    //         &node_stmt,
    //         &edge_stmt,
    //         &TreeNodeType::Dependent,
    //         depth,
    //     )
    //     .await?;

    //     Ok(TreeData { nodes, links })
    // }

    // #[async_recursion]
    // #[allow(clippy::too_many_arguments)]
    // async fn traverse_tree(
    //     db: &DbConn,
    //     pending_nodes: &mut Vec<String>,
    //     nodes: &mut Vec<TreeNodeData>,
    //     links: &mut Vec<LinkData>,
    //     node_stmt: &SelectStatement,
    //     edge_stmt: &SelectStatement,
    //     node_type: &TreeNodeType,
    //     depth: i32,
    // ) -> Result<(), DbErr> {
    //     if depth <= 0 || pending_nodes.is_empty() {
    //         return Ok(());
    //     }

    //     // let pending_nodes = dbg!(pending_nodes);

    //     let builder = db.get_database_backend();
    //     let mut node_stmt = node_stmt.clone();
    //     let target_nodes = mem::take(pending_nodes);
    //     node_stmt.and_where(Expr::col(Alias::new("name")).is_in(target_nodes));

    //     // dbg!(builder.build(&node_stmt));

    //     nodes.extend(
    //         Node::find_by_statement(builder.build(&node_stmt))
    //             .all(db)
    //             .await
    //             .map(|nodes| {
    //                 nodes
    //                     .into_iter()
    //                     .map(|node| {
    //                         pending_nodes.push(node.name.clone());
    //                         TreeNodeData {
    //                             id: node.name,
    //                             r#type: node_type.clone(),
    //                         }
    //                     })
    //                     .collect::<Vec<_>>()
    //             })?
    //             .into_iter(),
    //     );

    //     let mut edge_stmt = edge_stmt.clone();
    //     let from_nodes = mem::take(pending_nodes);
    //     // let from_nodes = dbg!(from_nodes);
    //     match node_type {
    //         TreeNodeType::Root => todo!(),
    //         TreeNodeType::Dependency => todo!(),
    //         TreeNodeType::Dependent => todo!(),
    //     }
    //     edge_stmt.and_where(Expr::col(Alias::new("from_node")).is_in(from_nodes));

    //     // dbg!(builder.build(&edge_stmt));

    //     let mut pending_nodes = Vec::new();
    //     links.extend(
    //         Link::find_by_statement(builder.build(&edge_stmt))
    //             .all(db)
    //             .await
    //             .map(|links| {
    //                 links
    //                     .into_iter()
    //                     .map(|edge| {
    //                         pending_nodes.push(edge.to_node.clone());
    //                         edge.into()
    //                     })
    //                     .collect::<Vec<_>>()
    //             })?
    //             .into_iter(),
    //     );

    //     // let mut pending_nodes = dbg!(pending_nodes);

    //     Self::traverse_tree(
    //         db,
    //         &mut pending_nodes,
    //         nodes,
    //         links,
    //         &node_stmt,
    //         &edge_stmt,
    //         node_type,
    //         depth - 1,
    //     )
    //     .await?;

    //     Ok(())
    // }
}
