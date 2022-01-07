//! Graph query engine

use async_recursion::async_recursion;
use sea_orm::{ConnectionTrait, DbConn, DbErr, FromQueryResult, Order, Statement};
use sea_query::{Alias, Expr, SelectStatement};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{cmp::min, collections::HashSet};

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
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
            .columns([Alias::new("name"), Alias::new("in_conn")])
            .from(Alias::new("node_crate"));
        let mut edge_stmt = sea_query::Query::select();
        edge_stmt
            .columns([Alias::new("from_node"), Alias::new("to_node")])
            .from(Alias::new("edge_depends"))
            .inner_join(
                Alias::new("node_crate"),
                Expr::tbl(Alias::new("node_crate"), Alias::new("name"))
                    .equals(Alias::new("edge_depends"), Alias::new("from_node")),
            )
            .order_by(
                (Alias::new("node_crate"), Alias::new("in_conn")),
                Order::Desc,
            );

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

        let mut new_pending_nodes = Vec::new();
        while !pending_nodes.is_empty() {
            let mut temp_pending_nodes = Vec::new();
            let len = min(100, pending_nodes.len());
            let drained_nodes = pending_nodes.drain(..len);
            let mut stmts = Vec::new();
            for node in drained_nodes {
                let mut stmt = edge_stmt.clone();
                stmt.and_where(Expr::col(Alias::new("to_node")).eq(node.as_str()))
                    .limit(limit as u64);
                let stmt = builder.build(&stmt).to_string();
                stmts.push(stmt);
            }
            let stmt = Statement::from_string(builder, format!("({})", stmts.join(") UNION (")));
            links.extend(
                Link::find_by_statement(stmt)
                    .all(db)
                    .await
                    .map(|links| {
                        links
                            .into_iter()
                            .map(|edge| {
                                temp_pending_nodes.push(edge.from_node.clone());
                                edge.into()
                            })
                            .collect::<Vec<_>>()
                    })?
                    .into_iter(),
            );

            let mut stmt = node_stmt.clone();
            stmt.and_where(Expr::col(Alias::new("name")).is_in(temp_pending_nodes.clone()));
            nodes.extend(
                Node::find_by_statement(builder.build(&stmt))
                    .all(db)
                    .await
                    .map(|nodes| nodes.into_iter().map(Into::into).collect::<Vec<_>>())?
                    .into_iter(),
            );
            new_pending_nodes.extend(temp_pending_nodes);
        }
        assert!(pending_nodes.is_empty());
        pending_nodes.extend(new_pending_nodes);

        if first {
            let mut stmt = node_stmt.clone();
            stmt.order_by(Alias::new("in_conn"), Order::Desc)
                .limit(top_n as u64);
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
        }

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

    /// Get tree
    pub async fn get_tree(
        db: &DbConn,
        root_node: String,
        limit: i32,
        depth: i32,
    ) -> Result<TreeData, DbErr> {
        let builder = db.get_database_backend();
        let mut pending_lib_nodes = vec![root_node.clone()];
        let mut pending_app_nodes = vec![root_node.clone()];
        let mut nodes = HashSet::new();
        let mut links = Vec::new();
        let mut node_stmt = sea_query::Query::select();
        node_stmt
            .columns([Alias::new("name"), Alias::new("in_conn")])
            .from(Alias::new("node_crate"));
        let mut edge_stmt = sea_query::Query::select();
        edge_stmt
            .columns([Alias::new("from_node"), Alias::new("to_node")])
            .from(Alias::new("edge_depends"));

        let mut stmt = node_stmt.clone();
        stmt.and_where(Expr::col(Alias::new("name")).eq(root_node.as_str()));
        let node = Node::find_by_statement(builder.build(&stmt))
            .one(db)
            .await?
            .ok_or_else(|| {
                DbErr::Custom(format!(
                    "Root node of name '{}' could not be found",
                    root_node
                ))
            })?;
        nodes.insert(TreeNodeData {
            id: node.name,
            r#type: TreeNodeType::Root,
        });

        if depth > 0 {
            Self::traverse_tree(
                db,
                &mut pending_lib_nodes,
                &mut nodes,
                &mut links,
                &node_stmt,
                &edge_stmt,
                &TreeNodeType::Dependency,
                limit,
                depth - 1,
            )
            .await?;
        }

        if depth > 0 {
            Self::traverse_tree(
                db,
                &mut pending_app_nodes,
                &mut nodes,
                &mut links,
                &node_stmt,
                &edge_stmt,
                &TreeNodeType::Dependent,
                limit,
                depth - 1,
            )
            .await?;
        }

        Ok(TreeData {
            nodes: nodes.into_iter().collect(),
            links,
        })
    }

    #[async_recursion]
    #[allow(clippy::too_many_arguments)]
    async fn traverse_tree(
        db: &DbConn,
        pending_nodes: &mut Vec<String>,
        nodes: &mut HashSet<TreeNodeData>,
        links: &mut Vec<LinkData>,
        node_stmt: &SelectStatement,
        edge_stmt: &SelectStatement,
        node_type: &TreeNodeType,
        limit: i32,
        depth: i32,
    ) -> Result<(), DbErr> {
        let builder = db.get_database_backend();

        let mut new_pending_nodes = Vec::new();
        while !pending_nodes.is_empty() {
            let mut temp_pending_nodes = Vec::new();
            let len = min(100, pending_nodes.len());
            let drained_nodes = pending_nodes.drain(..len);
            let mut stmts = Vec::new();
            for node in drained_nodes {
                let mut stmt = edge_stmt.clone();
                stmt.order_by(
                    (Alias::new("node_crate"), Alias::new("in_conn")),
                    Order::Desc,
                );
                match node_type {
                    TreeNodeType::Root => unreachable!(),
                    TreeNodeType::Dependency => stmt
                        .and_where(Expr::col(Alias::new("from_node")).eq(node.as_str()))
                        .inner_join(
                            Alias::new("node_crate"),
                            Expr::tbl(Alias::new("node_crate"), Alias::new("name"))
                                .equals(Alias::new("edge_depends"), Alias::new("to_node")),
                        ),
                    TreeNodeType::Dependent => stmt
                        .and_where(Expr::col(Alias::new("to_node")).eq(node.as_str()))
                        .inner_join(
                            Alias::new("node_crate"),
                            Expr::tbl(Alias::new("node_crate"), Alias::new("name"))
                                .equals(Alias::new("edge_depends"), Alias::new("from_node")),
                        ),
                };
                stmt.limit(limit as u64);
                let stmt = builder.build(&stmt).to_string();
                stmts.push(stmt);
            }
            let stmt = Statement::from_string(builder, format!("({})", stmts.join(") UNION (")));
            links.extend(
                Link::find_by_statement(stmt)
                    .all(db)
                    .await
                    .map(|links| {
                        links
                            .into_iter()
                            .map(|edge| {
                                match node_type {
                                    TreeNodeType::Root => unreachable!(),
                                    TreeNodeType::Dependency => {
                                        temp_pending_nodes.push(edge.to_node.clone())
                                    }
                                    TreeNodeType::Dependent => {
                                        temp_pending_nodes.push(edge.from_node.clone())
                                    }
                                }
                                edge.into()
                            })
                            .collect::<Vec<_>>()
                    })?
                    .into_iter(),
            );

            let mut stmt = node_stmt.clone();
            stmt.and_where(Expr::col(Alias::new("name")).is_in(temp_pending_nodes.clone()));
            nodes.extend(
                Node::find_by_statement(builder.build(&stmt))
                    .all(db)
                    .await
                    .map(|nodes| {
                        nodes
                            .into_iter()
                            .map(|node| TreeNodeData {
                                id: node.name,
                                r#type: node_type.clone(),
                            })
                            .collect::<Vec<_>>()
                    })?
                    .into_iter(),
            );
            new_pending_nodes.extend(temp_pending_nodes);
        }
        assert!(pending_nodes.is_empty());
        pending_nodes.extend(new_pending_nodes);

        if depth > 0 && !pending_nodes.is_empty() {
            Self::traverse_tree(
                db,
                pending_nodes,
                nodes,
                links,
                node_stmt,
                edge_stmt,
                node_type,
                limit,
                depth - 1,
            )
            .await?;
        }

        Ok(())
    }
}
