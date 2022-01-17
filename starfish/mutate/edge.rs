use std::collections::{HashMap, HashSet, VecDeque};

use super::Mutate;
use crate::schema::format_edge_table_name;
use sea_orm::{ConnectionTrait, DbConn, DbErr, DeriveIden, FromQueryResult, Statement};
use sea_query::{Alias, Expr, Query};
use serde::{Deserialize, Serialize};

/// Metadata of a edge, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EdgeJson {
    /// Name of relation
    pub name: String,
    /// Name of related node (from side)
    pub from_node: String,
    /// Name of related node (to side)
    pub to_node: String,
}

/// Metadata of edges in batch, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EdgeJsonBatch {
    /// Name of relation
    pub name: String,
    /// Vector of edges
    pub edges: Vec<Edge>,
}

/// Metadata of a edge in batch, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Edge {
    /// Name of related node (from side)
    pub from_node: String,
    /// Name of related node (to side)
    pub to_node: String,
}

/// Metadata of a edge, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClearEdgeJson {
    /// Name of relation
    pub name: String,
    /// Name of node
    pub node: String,
}

#[derive(Debug, Clone, FromQueryResult)]
struct Node {
    name: String,
}

#[derive(Debug, Clone, FromQueryResult)]
struct Link {
    from_node: String,
    to_node: String,
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

        Self::update_connectivity(db).await?;

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

        Self::update_connectivity(db).await?;

        Ok(())
    }

    /// Clear edge
    pub async fn clear_edge(db: &DbConn, clear_edge_json: ClearEdgeJson) -> Result<(), DbErr> {
        let mut stmt = Query::delete();
        stmt.from_table(Alias::new(&format_edge_table_name(clear_edge_json.name)))
            .and_where(Expr::col(Alias::new("from_node")).eq(clear_edge_json.node));

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Self::update_connectivity(db).await?;

        Ok(())
    }

    async fn update_connectivity(db: &DbConn) -> Result<(), DbErr> {
        db.execute(Statement::from_string(db.get_database_backend(), [
            "UPDATE node_crate",
            "SET in_conn = (SELECT COUNT(*) FROM edge_depends WHERE to_node = node_crate.name),",
            "out_conn = (SELECT COUNT(*) FROM edge_depends WHERE from_node = node_crate.name)",
        ].join(" "))).await?;

        Ok(())
    }

    /// Update compound connectivity
    pub async fn calculate_compound_connectivity(db: &DbConn) -> Result<(), DbErr> {
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
                        .push(link.from_node);

                    node_to_parents
                })
        };

        let mut map_id_to_ancestors: HashMap<String, HashSet<String>> = HashMap::new();

        for (i, root_node) in nodes.into_iter().enumerate() {
            if (i + 1) % 1000 == 0 || i + 1 == num_nodes {
                println!("Handling root node {}/{}", i + 1, num_nodes);
            }
            let mut queue: VecDeque<String> = VecDeque::new();
            queue.push_back(root_node.name.clone());

            let mut ancestors = HashSet::new();

            while !queue.is_empty() {
                let current_node = queue.pop_front().unwrap();
                if let Some(current_parents) = node_to_parents.get(&current_node) {
                    for parent in current_parents {
                        // Previously used subpaths within current node are avoided
                        if ancestors.contains(parent) {
                            continue;
                        }
                        ancestors.insert(parent.clone());
                        // Dynamic programming: reuse previously obtained ancestors
                        if let Some(parent_ancestors) = map_id_to_ancestors.get(parent) {
                            ancestors.extend(parent_ancestors.clone());
                        } else {
                            queue.push_back(parent.clone());
                        }
                    }
                }
            }

            map_id_to_ancestors.insert(root_node.name, ancestors);
        }

        // map_id_to_ancestors is ready; the sizes of the sets in its values are the compound in_conn
        let cols = [Alias::new("name"), Alias::new("in_conn_compound")];
        let mut stmt = Query::insert();
        stmt.into_table(Alias::new("node_crate"))
            .columns(cols.clone());

        for (name, ancestor_names) in map_id_to_ancestors.into_iter() {
            let in_conn_compound = ancestor_names.len() as i32;
            stmt.values_panic([name.into(), in_conn_compound.into()]);
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
