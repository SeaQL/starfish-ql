use std::collections::{HashMap, HashSet, VecDeque};

use super::Mutate;
use crate::{
    lang::{ClearEdgeJson, Edge, EdgeJson, EdgeJsonBatch},
    schema::format_edge_table_name,
};
use sea_orm::{ConnectionTrait, DbConn, DbErr, DeriveIden, FromQueryResult, Statement};
use sea_query::{Alias, Expr, Query};

#[derive(Debug, Clone, FromQueryResult)]
struct Node {
    name: String,
}

#[derive(Debug, Clone, FromQueryResult)]
struct Link {
    from_node: String,
    to_node: String,
}

#[derive(Debug)]
struct NodeAncestor {
    name: String,
    weight: f64,
}

impl PartialEq for NodeAncestor {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for NodeAncestor {}

impl std::hash::Hash for NodeAncestor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl NodeAncestor {
    fn clone_with_new_weight(&self, new_weight: f64) -> Self {
        Self {
            name: self.name.clone(),
            weight: new_weight,
        }
    }
}

impl Mutate {
    /// Insert edge
    pub async fn insert_edge(db: &DbConn, edge_json: EdgeJson) -> Result<(), DbErr> {
        Self::insert_edge_batch(
            db,
            EdgeJsonBatch {
                name: edge_json.name,
                edges: vec![Edge {
                    from_node: edge_json.from_node,
                    to_node: edge_json.to_node,
                }],
            },
        )
        .await
    }

    /// Insert edge in batch
    pub async fn insert_edge_batch(
        db: &DbConn,
        edge_json_batch: EdgeJsonBatch,
    ) -> Result<(), DbErr> {
        let mut stmt = Query::insert();
        stmt.into_table(Alias::new(&format_edge_table_name(edge_json_batch.name)))
            .columns([Alias::new("from_node"), Alias::new("to_node")]);

        for edge_json in edge_json_batch.edges.into_iter() {
            stmt.values_panic([edge_json.from_node.into(), edge_json.to_node.into()]);
        }

        let builder = db.get_database_backend();
        let mut stmt = builder.build(&stmt);
        stmt.sql = stmt.sql.replace("INSERT", "INSERT IGNORE");
        db.execute(stmt).await?;

        Ok(())
    }

    /// Delete edge
    pub async fn delete_edge(db: &DbConn, edge_json: EdgeJson) -> Result<(), DbErr> {
        let mut stmt = Query::delete();
        stmt.from_table(Alias::new(&format_edge_table_name(edge_json.name)))
            .and_where(Expr::col(Alias::new("from_node")).eq(edge_json.from_node))
            .and_where(Expr::col(Alias::new("to_node")).eq(edge_json.to_node));

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }

    /// Clear edge
    pub async fn clear_edge(db: &DbConn, clear_edge_json: ClearEdgeJson) -> Result<(), DbErr> {
        let mut stmt = Query::delete();
        stmt.from_table(Alias::new(&format_edge_table_name(clear_edge_json.name)))
            .and_where(Expr::col(Alias::new("from_node")).eq(clear_edge_json.node));

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }

    pub async fn calculate_simple_connectivity(db: &DbConn) -> Result<(), DbErr> {
        db.execute(Statement::from_string(db.get_database_backend(), [
            "UPDATE node_crate",
            "SET in_conn = (SELECT COUNT(*) FROM edge_depends WHERE to_node = node_crate.name),",
            "out_conn = (SELECT COUNT(*) FROM edge_depends WHERE from_node = node_crate.name)",
        ].join(" "))).await?;

        Ok(())
    }

    pub async fn calculate_compound_connectivity(db: &DbConn) -> Result<(), DbErr> {
        Self::calculate_complex_connectivity(db, 1.0, f64::EPSILON, "in_conn_compound").await
    }

    pub async fn calculate_complex_connectivity(
        db: &DbConn,
        weight: f64,
        epsilon: f64,
        col_name: &str,
    ) -> Result<(), DbErr> {
        let builder = db.get_database_backend();
        let mut node_stmt = sea_query::Query::select();
        node_stmt
            .column(Alias::new("name"))
            .from(Alias::new("node_crate"))
            .and_where(Expr::col(Alias::new("in_conn")).gt(0));
        let nodes = Node::find_by_statement(builder.build(&node_stmt))
            .all(db)
            .await?;
        let num_nodes = nodes.len();

        let node_to_parents = {
            (Link::find_by_statement(
                builder.build(
                    sea_query::Query::select()
                        .columns([Alias::new("from_node"), Alias::new("to_node")])
                        .from(Alias::new("edge_depends")),
                ),
            )
            .all(db)
            .await?)
                .into_iter()
                .fold(HashMap::new(), |mut node_to_parents, link| {
                    if !node_to_parents.contains_key(&link.to_node) {
                        node_to_parents.insert(link.to_node.clone(), vec![]);
                    }
                    node_to_parents
                        .get_mut(&link.to_node)
                        .unwrap()
                        .push(NodeAncestor {
                            name: link.from_node,
                            weight: f64::NAN,
                        }); // This weight is not meant to be used

                    node_to_parents
                })
        };

        let mut map_id_to_ancestors: HashMap<String, HashSet<NodeAncestor>> = HashMap::new();

        for (i, root_node) in nodes.into_iter().enumerate() {
            if (i + 1) % 1000 == 0 || i + 1 == num_nodes {
                println!("Handling root node {}/{}", i + 1, num_nodes);
            }
            let mut queue: VecDeque<Option<String>> = VecDeque::new();
            queue.push_back(Some(root_node.name.clone()));
            queue.push_back(None); // Use None to separate each level

            let mut ancestors = HashSet::new();
            let mut current_weight = 1.0;

            while !queue.is_empty() {
                if let Some(current_node) = queue.pop_front().unwrap() {
                    if let Some(current_parents) = node_to_parents.get(&current_node) {
                        for parent in current_parents {
                            // Previously used subpaths within current node are avoided
                            // because they are shortest paths already
                            if ancestors.contains(parent) {
                                continue;
                            }
                            ancestors.insert(parent.clone_with_new_weight(current_weight));
                            if let Some(parent_ancestors) = map_id_to_ancestors.get(&parent.name) {
                                // Dynamic programming: reuse previously obtained ancestors
                                for parent_ancestor in parent_ancestors {
                                    let parent_ancestor_weight =
                                        parent_ancestor.weight * current_weight * weight;
                                    if parent_ancestor_weight > epsilon {
                                        ancestors.insert(
                                            parent_ancestor
                                                .clone_with_new_weight(parent_ancestor_weight),
                                        );
                                    }
                                }
                            } else {
                                queue.push_back(Some(parent.name.clone()));
                            }
                        }
                    }
                } else {
                    // End of level
                    current_weight *= weight;
                    if current_weight <= epsilon {
                        break;
                    }
                    // If queue is empty, all ancestor nodes are visited
                    if !queue.is_empty() {
                        queue.push_back(None);
                    }
                }
            }

            map_id_to_ancestors.insert(root_node.name, ancestors);
        }

        // map_id_to_ancestors is ready; the sizes of the sets in its values are the compound in_conn
        let cols = [Alias::new("name"), Alias::new(col_name)];
        let mut stmt = Query::insert();
        stmt.into_table(Alias::new("node_crate"))
            .columns(cols.clone());

        for (name, ancestors) in map_id_to_ancestors.into_iter() {
            let in_conn_complex = ancestors
                .into_iter()
                .fold(0.0, |conn, ancestor| conn + ancestor.weight);
            stmt.values_panic([name.into(), in_conn_complex.into()]);
        }

        let update_vals = cols
            .into_iter()
            .map(|col| {
                let col = col.to_string();
                format!("{0} = VALUES({0})", col)
            })
            .collect::<Vec<_>>()
            .join(", ");
        let builder = db.get_database_backend();
        let mut stmt = builder.build(&stmt);
        stmt.sql = format!("{} ON DUPLICATE KEY UPDATE {}", stmt.sql, update_vals);
        db.execute(stmt).await?;

        Ok(())
    }
}
