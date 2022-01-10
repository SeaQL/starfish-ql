//! Graph query engine

use async_recursion::async_recursion;
use rocket::futures::executor;
use sea_orm::{ConnectionTrait, DbConn, DbErr, FromQueryResult, Order, Statement};
use sea_query::{Alias, Expr, SelectStatement};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::mem;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::{
    cmp::min,
    collections::{HashSet, VecDeque},
    thread,
};

const BATCH_SIZE: usize = 100;
const DEBUG: bool = false;

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

#[derive(Debug, Clone, FromQueryResult)]
struct Node {
    name: String,
    in_conn: i32,
}

#[derive(Debug, Clone, FromQueryResult)]
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

#[derive(Debug)]
enum ExecutorMsg {
    Execute {
        depth: i32,
        pending_nodes: Vec<String>,
    },
    Quit,
}

#[derive(Debug)]
enum ResultMsg {
    Done {
        depth: i32,
        nodes: HashSet<GraphNodeData>,
        links: HashSet<LinkData>,
    },
}

fn start_executor(
    i: usize,
    db: DbConn,
    limit: i32,
    result_sender: Sender<ResultMsg>,
) -> Sender<ExecutorMsg> {
    let (executor_sender, executor_receiver) = channel();
    let _ = thread::Builder::new().spawn(move || {
        if DEBUG {
            println!("Thread {} | spawning thread", i);
        }
        let mut executor = WorkflowExecutor {
            i,
            db,
            limit,
            tasks: Default::default(),
            executor_receiver,
            result_sender,
        };
        while executor.run() {
            // Running...
        }
    });
    executor_sender
}

struct WorkflowExecutor {
    i: usize,
    db: DbConn,
    limit: i32,
    tasks: VecDeque<(i32, Vec<String>)>,
    executor_receiver: Receiver<ExecutorMsg>,
    result_sender: Sender<ResultMsg>,
}

impl WorkflowExecutor {
    fn handle_msg(&mut self) -> bool {
        match self.executor_receiver.try_recv() {
            Ok(ExecutorMsg::Execute {
                depth,
                pending_nodes,
            }) => {
                if DEBUG {
                    println!(
                        "Thread {} | received task\ndepth: {}, nodes: {:#?}",
                        self.i, depth, pending_nodes
                    );
                }
                self.tasks.push_back((depth, pending_nodes));
            }
            Ok(ExecutorMsg::Quit) => {
                if DEBUG {
                    println!("Thread {} | received quit signal", self.i);
                }
                return false;
            }
            Err(_) => {}
        }
        true
    }

    fn execute_task(&mut self) {
        if let Some((depth, pending_nodes)) = self.tasks.pop_front() {
            if DEBUG {
                println!(
                    "Thread {} | execute task\ndepth: {}, nodes: {:#?}",
                    self.i, depth, pending_nodes
                );
            }
            executor::block_on(async {
                let mut nodes = Default::default();
                let mut links = Default::default();

                Query::traverse_graph(
                    &self.db,
                    pending_nodes,
                    &mut nodes,
                    &mut links,
                    0,
                    self.limit,
                    false,
                )
                .await
                .unwrap();

                self.result_sender
                    .send(ResultMsg::Done {
                        depth,
                        nodes,
                        links,
                    })
                    .unwrap();
            });
        }
    }

    fn run(&mut self) -> bool {
        if !self.handle_msg() {
            return false;
        }
        self.execute_task();
        true
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
        let mut nodes = HashSet::new();
        let mut links = HashSet::new();

        Self::traverse_graph(db, vec![], &mut nodes, &mut links, top_n, limit, true).await?;

        if depth <= 0 {
            return Ok(GraphData {
                nodes: nodes.into_iter().collect(),
                links: links.into_iter().collect(),
            });
        }

        let (results_sender, results_receiver) = channel();

        let mut executors: VecDeque<Sender<ExecutorMsg>> = vec![
            start_executor(1, db.clone(), limit, results_sender.clone()),
            start_executor(2, db.clone(), limit, results_sender.clone()),
            start_executor(3, db.clone(), limit, results_sender.clone()),
            start_executor(4, db.clone(), limit, results_sender),
        ]
        .into_iter()
        .collect();

        let mut pending_tasks: VecDeque<ExecutorMsg> = vec![ExecutorMsg::Execute {
            depth,
            pending_nodes: nodes.iter().map(|node| node.id.clone()).collect(),
        }]
        .into_iter()
        .collect();
        let mut running_tasks = 0u32;

        loop {
            if let Some(task) = pending_tasks.pop_front() {
                if DEBUG {
                    println!("Main | sending task: {:#?}", task);
                }
                if let Some(executor) = executors.pop_front() {
                    if executor.send(task).is_ok() {
                        executors.push_back(executor);
                        running_tasks += 1;
                    }
                }
            }
            if let Ok(ResultMsg::Done {
                depth,
                nodes: rev_nodes,
                links: rev_links,
            }) = results_receiver.try_recv()
            {
                if DEBUG {
                    println!(
                        "Main | received result\nnodes: {:#?}\nlinks: {:#?}",
                        rev_nodes, rev_links
                    );
                }
                if depth > 0 {
                    let mut rev_nodes_clone = rev_nodes.iter().collect::<Vec<_>>();
                    while !rev_nodes_clone.is_empty() {
                        let len = min(BATCH_SIZE, rev_nodes_clone.len());
                        pending_tasks.push_back(ExecutorMsg::Execute {
                            depth: depth - 1,
                            pending_nodes: rev_nodes_clone
                                .drain(..len)
                                .map(|node| node.id.clone())
                                .collect(),
                        });
                    }
                }
                nodes.extend(rev_nodes);
                links.extend(rev_links);
                running_tasks -= 1;
            }
            if pending_tasks.is_empty() && running_tasks == 0 {
                for executor in executors {
                    executor.send(ExecutorMsg::Quit).unwrap();
                }
                break;
            }
        }

        Ok(GraphData {
            nodes: nodes.into_iter().collect(),
            links: links.into_iter().collect(),
        })
    }

    #[async_recursion]
    #[allow(clippy::too_many_arguments)]
    async fn traverse_graph(
        db: &DbConn,
        mut pending_nodes: Vec<String>,
        nodes: &mut HashSet<GraphNodeData>,
        links: &mut HashSet<LinkData>,
        top_n: i32,
        limit: i32,
        first: bool,
    ) -> Result<(), DbErr> {
        let builder = db.get_database_backend();
        let mut new_nodes = Vec::new();
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

        assert!(pending_nodes.len() <= BATCH_SIZE);

        if !pending_nodes.is_empty() {
            let mut stmts = Vec::new();
            for node in mem::take(&mut pending_nodes) {
                let mut stmt = edge_stmt.clone();
                stmt.and_where(Expr::col(Alias::new("to_node")).eq(node))
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
                                new_nodes.push(edge.from_node.clone());
                                edge.into()
                            })
                            .collect::<Vec<_>>()
                    })?
                    .into_iter(),
            );

            let mut stmt = node_stmt.clone();
            stmt.and_where(Expr::col(Alias::new("name")).is_in(mem::take(&mut new_nodes)));
            nodes.extend(
                Node::find_by_statement(builder.build(&stmt))
                    .all(db)
                    .await
                    .map(|nodes| nodes.into_iter().map(Into::into).collect::<Vec<_>>())?
                    .into_iter(),
            );
        }
        assert!(pending_nodes.is_empty());

        if first {
            let mut stmt = node_stmt.clone();
            stmt.order_by(Alias::new("in_conn"), Order::Desc)
                .limit(top_n as u64);
            nodes.extend(
                Node::find_by_statement(builder.build(&stmt))
                    .all(db)
                    .await
                    .map(|nodes| nodes.into_iter().map(Into::into).collect::<Vec<_>>())?
                    .into_iter(),
            );
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
