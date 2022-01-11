use super::{
    into_graph_link, into_graph_node, into_tree_link, into_tree_node, select_root_node,
    select_top_n_edge, select_top_n_node, traverse,
    worker::{TraverseData, TraverseType, Worker, WorkerMsg},
    GraphData, GraphLinkData, GraphNodeData, TreeData, TreeLinkData, TreeNodeData, TreeNodeType,
    BATCH_SIZE, DEBUG,
};
use sea_orm::{DbConn, DbErr};
use std::{
    collections::{HashSet, VecDeque},
    sync::mpsc::{channel, Receiver, Sender},
};

#[derive(Debug)]
pub(crate) enum ExecutorMsg {
    DoneGraph {
        depth: i32,
        nodes: Vec<GraphNodeData>,
        links: Vec<GraphLinkData>,
    },
    DoneTree {
        depth: i32,
        tree_node_type: TreeNodeType,
        nodes: Vec<TreeNodeData>,
        links: Vec<TreeLinkData>,
    },
}

pub(crate) struct Executor {
    db: DbConn,
    running_tasks: u32,
    pending_tasks: VecDeque<WorkerMsg>,
    workers: VecDeque<Sender<WorkerMsg>>,
    results_receiver: Receiver<ExecutorMsg>,
}

impl Executor {
    pub(crate) fn new(db: &DbConn) -> Self {
        let (results_sender, results_receiver) = channel();
        let workers = Self::spawn_workers(db, results_sender);

        Self {
            db: db.clone(),
            running_tasks: 0,
            pending_tasks: Default::default(),
            workers,
            results_receiver,
        }
    }

    fn spawn_workers(
        db: &DbConn,
        results_sender: Sender<ExecutorMsg>,
    ) -> VecDeque<Sender<WorkerMsg>> {
        let num_cups = num_cpus::get();
        (1..=num_cups)
            .collect::<Vec<_>>()
            .into_iter()
            .map(|i| Worker::spawn(i, db.clone(), results_sender.clone()))
            .collect()
    }

    fn all_tasks_completed(&self) -> bool {
        self.pending_tasks.is_empty() && self.running_tasks == 0
    }

    fn quit_all_workers(&mut self) {
        for worker in self.workers.iter() {
            worker.send(WorkerMsg::Quit).unwrap();
        }
    }

    pub(crate) async fn get_graph(
        &mut self,
        top_n: i32,
        limit: i32,
        depth: i32,
    ) -> Result<GraphData, DbErr> {
        let mut nodes = HashSet::new();
        let mut links = HashSet::new();

        let (res_nodes, res_links) = traverse(
            &self.db,
            TreeNodeType::Root,
            |node_stmt| select_top_n_node(node_stmt, top_n),
            |link_stmt| select_top_n_edge(link_stmt, limit, vec![], TreeNodeType::Root),
            into_graph_node,
            into_graph_link,
        )
        .await?;

        nodes.extend(res_nodes);
        links.extend(res_links);

        if nodes.is_empty() || depth <= 0 {
            return Ok(GraphData {
                nodes: nodes.into_iter().collect(),
                links: links.into_iter().collect(),
            });
        }

        self.pending_tasks.push_back(WorkerMsg::Traverse {
            data: TraverseData {
                limit,
                depth,
                pending_nodes: nodes.iter().map(|node| node.id.clone()).collect(),
                traverse_type: TraverseType::Graph,
            },
        });

        while !self.all_tasks_completed() {
            if let Some(task) = self.pending_tasks.pop_front() {
                if DEBUG {
                    println!("Main | sending task: {:#?}", task);
                }
                if let Some(worker) = self.workers.pop_front() {
                    if worker.send(task).is_ok() {
                        self.workers.push_back(worker);
                        self.running_tasks += 1;
                    }
                }
            }
            if let Ok(ExecutorMsg::DoneGraph {
                depth,
                nodes: rev_nodes,
                links: rev_links,
            }) = self.results_receiver.try_recv()
            {
                if DEBUG {
                    println!(
                        "Main | received result\nnodes: {:#?}\nlinks: {:#?}",
                        rev_nodes, rev_links
                    );
                }
                if depth > 1 {
                    let current_limit = std::cmp::max(
                        1,
                        (0..depth).fold(limit as f32, |current_limit, _| current_limit / 3.0 * 2.0)
                            as i32,
                    );
                    let mut rev_nodes_clone = rev_nodes.iter().collect::<Vec<_>>();
                    while !rev_nodes_clone.is_empty() {
                        let len = std::cmp::min(BATCH_SIZE, rev_nodes_clone.len());
                        self.pending_tasks.push_back(WorkerMsg::Traverse {
                            data: TraverseData {
                                limit: current_limit,
                                depth: depth - 1,
                                pending_nodes: rev_nodes_clone
                                    .drain(..len)
                                    .map(|node| node.id.clone())
                                    .collect(),
                                traverse_type: TraverseType::Graph,
                            },
                        });
                    }
                }
                nodes.extend(rev_nodes);
                links.extend(rev_links);
                self.running_tasks -= 1;
            }
        }
        self.quit_all_workers();

        Ok(GraphData {
            nodes: nodes.into_iter().collect(),
            links: links.into_iter().collect(),
        })
    }

    pub(crate) async fn get_tree(
        &mut self,
        root_node: String,
        limit: i32,
        depth: i32,
    ) -> Result<TreeData, DbErr> {
        let mut nodes = HashSet::new();
        let mut links = Vec::new();

        let (res_nodes, res_links) = traverse(
            &self.db,
            TreeNodeType::Root,
            |node_stmt| select_root_node(node_stmt, root_node),
            |link_stmt| select_top_n_edge(link_stmt, limit, vec![], TreeNodeType::Root),
            |node| into_tree_node(node, TreeNodeType::Root),
            |link| into_tree_link(link, TreeNodeType::Root),
        )
        .await?;

        nodes.extend(res_nodes);
        links.extend(res_links);

        if nodes.is_empty() || depth <= 0 {
            return Ok(TreeData {
                nodes: nodes.into_iter().collect(),
                links,
            });
        }

        self.pending_tasks.extend([
            WorkerMsg::Traverse {
                data: TraverseData {
                    limit,
                    depth,
                    pending_nodes: nodes.iter().map(|node| node.id.clone()).collect(),
                    traverse_type: TraverseType::Tree {
                        tree_node_type: TreeNodeType::Dependency,
                    },
                },
            },
            WorkerMsg::Traverse {
                data: TraverseData {
                    limit,
                    depth,
                    pending_nodes: nodes.iter().map(|node| node.id.clone()).collect(),
                    traverse_type: TraverseType::Tree {
                        tree_node_type: TreeNodeType::Dependent,
                    },
                },
            },
        ]);

        while !self.all_tasks_completed() {
            if let Some(task) = self.pending_tasks.pop_front() {
                if DEBUG {
                    println!("Main | sending task: {:#?}", task);
                }
                if let Some(worker) = self.workers.pop_front() {
                    if worker.send(task).is_ok() {
                        self.workers.push_back(worker);
                        self.running_tasks += 1;
                    }
                }
            }
            if let Ok(ExecutorMsg::DoneTree {
                depth,
                nodes: rev_nodes,
                links: rev_links,
                tree_node_type,
            }) = self.results_receiver.try_recv()
            {
                if DEBUG {
                    println!(
                        "Main | received result\nnodes: {:#?}\nlinks: {:#?}",
                        rev_nodes, rev_links
                    );
                }
                if depth > 1 {
                    let current_limit = std::cmp::max(
                        1,
                        (0..depth).fold(limit as f32, |current_limit, _| current_limit / 3.0 * 2.0)
                            as i32,
                    );
                    let mut rev_nodes_clone = rev_nodes.iter().collect::<Vec<_>>();
                    while !rev_nodes_clone.is_empty() {
                        let len = std::cmp::min(BATCH_SIZE, rev_nodes_clone.len());
                        self.pending_tasks.push_back(WorkerMsg::Traverse {
                            data: TraverseData {
                                limit: current_limit,
                                depth: depth - 1,
                                pending_nodes: rev_nodes_clone
                                    .drain(..len)
                                    .map(|node| node.id.clone())
                                    .collect(),
                                traverse_type: TraverseType::Tree { tree_node_type },
                            },
                        });
                    }
                }
                nodes.extend(rev_nodes);
                links.extend(rev_links);
                self.running_tasks -= 1;
            }
        }
        self.quit_all_workers();

        Ok(TreeData {
            nodes: nodes.into_iter().collect(),
            links,
        })
    }
}
