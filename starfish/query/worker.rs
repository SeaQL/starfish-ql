use super::{
    executor::ExecutorMsg, into_graph_link, into_graph_node, into_tree_link, into_tree_node,
    select_top_n_edge, traverse, TreeNodeType, DEBUG,
};
use rocket::futures::executor::block_on;
use sea_orm::DbConn;
use std::{
    collections::VecDeque,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

#[derive(Debug)]
pub(crate) enum WorkerMsg {
    Traverse { data: TraverseData },
    Quit,
}

#[derive(Debug)]
pub(crate) struct TraverseData {
    pub(crate) limit: i32,
    pub(crate) depth: i32,
    pub(crate) pending_nodes: Vec<String>,
    pub(crate) traverse_type: TraverseType,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TraverseType {
    Graph,
    Tree { tree_node_type: TreeNodeType },
}

pub(crate) struct Worker {
    i: usize,
    db: DbConn,
    tasks: VecDeque<TraverseData>,
    worker_receiver: Receiver<WorkerMsg>,
    result_sender: Sender<ExecutorMsg>,
}

impl Worker {
    pub(crate) fn spawn(
        i: usize,
        db: DbConn,
        result_sender: Sender<ExecutorMsg>,
    ) -> Sender<WorkerMsg> {
        let (worker_sender, worker_receiver) = channel();
        let _ = thread::Builder::new().spawn(move || {
            if DEBUG {
                println!("Thread {} | spawning thread", i);
            }
            let mut worker = Worker {
                i,
                db,
                tasks: Default::default(),
                worker_receiver,
                result_sender,
            };
            while worker.run() {
                // Running...
            }
        });
        worker_sender
    }

    fn handle_msg(&mut self) -> bool {
        match self.worker_receiver.try_recv() {
            Ok(WorkerMsg::Traverse { data }) => {
                if DEBUG {
                    println!("Thread {} | received task: {:#?}", self.i, data);
                }
                self.tasks.push_back(data);
            }
            Ok(WorkerMsg::Quit) => {
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
        if let Some(TraverseData {
            limit,
            depth,
            pending_nodes,
            traverse_type,
        }) = self.tasks.pop_front()
        {
            if DEBUG {
                println!(
                    "Thread {} | execute task\ndepth: {}, nodes: {:#?}",
                    self.i, depth, pending_nodes
                );
            }
            block_on(async {
                match traverse_type {
                    TraverseType::Graph => {
                        let (nodes, links) = traverse(
                            &self.db,
                            TreeNodeType::Dependency,
                            |_| unreachable!(),
                            |link_stmt| {
                                select_top_n_edge(
                                    link_stmt,
                                    limit,
                                    pending_nodes,
                                    TreeNodeType::Dependency,
                                )
                            },
                            into_graph_node,
                            into_graph_link,
                        )
                        .await
                        .unwrap();

                        self.result_sender
                            .send(ExecutorMsg::DoneGraph {
                                depth,
                                nodes,
                                links,
                            })
                            .unwrap();
                    }
                    TraverseType::Tree { tree_node_type } => {
                        let (nodes, links) = traverse(
                            &self.db,
                            tree_node_type,
                            |_| unreachable!(),
                            |link_stmt| {
                                select_top_n_edge(link_stmt, limit, pending_nodes, tree_node_type)
                            },
                            |node| into_tree_node(node, tree_node_type, depth),
                            |link| into_tree_link(link, tree_node_type),
                        )
                        .await
                        .unwrap();

                        self.result_sender
                            .send(ExecutorMsg::DoneTree {
                                depth,
                                nodes,
                                links,
                                tree_node_type,
                            })
                            .unwrap();
                    }
                }
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
