use std::collections::{HashMap, HashSet, VecDeque};

use super::Mutate;
use crate::{
    entities::{relation::Model, Relation},
    lang::{
        iden::{EdgeIden, NodeIden},
        mutate::{MutateEdgeContentJson, MutateEdgeSelectorJson},
        ClearEdgeJson, Edge, EdgeJson, EdgeJsonBatch,
    },
    schema::{format_edge_table_name, format_node_table_name},
};
use sea_orm::{ConnectionTrait, DbConn, DbErr, EntityTrait, FromQueryResult, Value};
use sea_query::{
    Alias, Cond, Expr, IntoIden, OnConflict, Query, QueryStatementBuilder, SimpleExpr,
};

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
                of: edge_json.name,
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
        stmt.into_table(Alias::new(&format_edge_table_name(edge_json_batch.of)))
            .columns([EdgeIden::FromNode, EdgeIden::ToNode]);

        for edge_json in edge_json_batch.edges.into_iter() {
            stmt.values_panic([edge_json.from_node.into(), edge_json.to_node.into()]);
        }

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }

    /// Delete edge
    pub async fn delete_edge(db: &DbConn, edge_json: EdgeJson) -> Result<(), DbErr> {
        let mut stmt = Query::delete();
        stmt.from_table(Alias::new(&format_edge_table_name(edge_json.name)))
            .and_where(Expr::col(EdgeIden::FromNode).eq(edge_json.from_node))
            .and_where(Expr::col(EdgeIden::ToNode).eq(edge_json.to_node));

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }

    /// Clear edge
    pub async fn clear_edge(db: &DbConn, clear_edge_json: ClearEdgeJson) -> Result<(), DbErr> {
        let mut stmt = Query::delete();
        stmt.from_table(Alias::new(&format_edge_table_name(clear_edge_json.name)))
            .and_where(Expr::col(EdgeIden::FromNode).eq(clear_edge_json.node));

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }

    /// Delete edge with selector
    pub async fn delete_edge_with_selector(
        db: &DbConn,
        selector: MutateEdgeSelectorJson,
    ) -> Result<(), DbErr> {
        let mut condition = Cond::all();
        if let Some(from_node) = selector.edge_content.from_node {
            condition = condition.add(Expr::col(EdgeIden::FromNode).eq(from_node));
        }
        if let Some(to_node) = selector.edge_content.to_node {
            condition = condition.add(Expr::col(EdgeIden::ToNode).eq(to_node));
        }

        let stmt = Query::delete()
            .from_table(Alias::new(&format_edge_table_name(selector.of)))
            .cond_where(condition)
            .to_owned();

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }

    /// Update edge
    pub async fn update_edge(
        db: &DbConn,
        selector: MutateEdgeSelectorJson,
        content: MutateEdgeContentJson,
    ) -> Result<(), DbErr> {
        let mut set_values: Vec<(EdgeIden, Value)> = vec![];
        if let Some(from_node) = content.from_node {
            set_values.push((EdgeIden::FromNode, from_node.into()));
        }
        if let Some(to_node) = content.to_node {
            set_values.push((EdgeIden::ToNode, to_node.into()));
        }
        if set_values.is_empty() {
            return Ok(());
        }

        let mut condition = Cond::all();
        if let Some(from_node) = selector.edge_content.from_node {
            condition = condition.add(Expr::col(EdgeIden::FromNode).eq(from_node));
        }
        if let Some(to_node) = selector.edge_content.to_node {
            condition = condition.add(Expr::col(EdgeIden::ToNode).eq(to_node));
        }

        let stmt = Query::update()
            .table(Alias::new(&format_edge_table_name(selector.of)))
            .values(set_values)
            .cond_where(condition)
            .to_owned();

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }

    /// Calculate the connectivity of all relations specified by the supplied *unformatted* names
    pub async fn calculate_all_connectivity(
        db: &DbConn,
        relation_names: Vec<String>,
    ) -> Result<(), DbErr> {
        let mut relation_names: HashSet<String> = HashSet::from_iter(relation_names.into_iter());

        let relations = Relation::find()
            .all(db)
            .await?
            .into_iter()
            .filter(|relation| relation_names.remove(&relation.name))
            .collect::<Vec<Model>>();

        if !relation_names.is_empty() {
            let missing_relation_names = Vec::from_iter(relation_names.into_iter()).join(", ");
            return Err(DbErr::Custom(format!(
                "Relations not found: {}",
                missing_relation_names
            )));
        }

        for relation in relations {
            let relation_name = relation.name.as_str();
            let from_node = relation.from_entity.as_str();
            let to_node = relation.to_entity.as_str();

            Self::calculate_simple_connectivity(db, relation_name, from_node, to_node).await?;

            Self::calculate_compound_connectivity(db, relation_name, to_node).await?;

            for (weight, col_name) in [
                (0.3, "in_conn_complex03"),
                (0.5, "in_conn_complex05"),
                (0.7, "in_conn_complex07"),
            ] {
                Self::calculate_complex_connectivity(
                    db,
                    relation_name,
                    to_node,
                    weight,
                    f64::EPSILON,
                    col_name,
                )
                .await?;
            }
        }

        Ok(())
    }

    /// Calculate simple incoming and outgoing connectivity
    pub async fn calculate_simple_connectivity(
        db: &DbConn,
        relation_name: &str,
        from_node: &str,
        to_node: &str,
    ) -> Result<(), DbErr> {
        let builder = db.get_database_backend();
        let edge_table = &format_edge_table_name(relation_name);

        // Calculate the simple outgoing connectivity value
        let node_table = &format_node_table_name(from_node);
        let mut select = Query::select();
        select
            .from(Alias::new(edge_table))
            .expr(Expr::cust("COUNT(*)"))
            .and_where(
                Expr::col(EdgeIden::FromNode).equals(Alias::new(node_table), NodeIden::Name),
            );
        let mut stmt = Query::update();
        stmt.table(Alias::new(node_table)).value_expr(
            Alias::new(&format!("{}_out_conn", relation_name)),
            SimpleExpr::SubQuery(Box::new(select.into_sub_query_statement())),
        );
        db.execute(builder.build(&stmt)).await?;

        // Calculate the simple incoming connectivity value
        let node_table = &format_node_table_name(to_node);
        let mut select = Query::select();
        select
            .from(Alias::new(edge_table))
            .expr(Expr::cust("COUNT(*)"))
            .and_where(Expr::col(EdgeIden::ToNode).equals(Alias::new(node_table), NodeIden::Name));
        let mut stmt = Query::update();
        stmt.table(Alias::new(node_table)).value_expr(
            Alias::new(&format!("{}_in_conn", relation_name)),
            SimpleExpr::SubQuery(Box::new(select.into_sub_query_statement())),
        );
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }

    /// Calculate compound incoming connectivity
    pub async fn calculate_compound_connectivity(
        db: &DbConn,
        relation_name: &str,
        to_node: &str,
    ) -> Result<(), DbErr> {
        Self::calculate_complex_connectivity(
            db,
            relation_name,
            to_node,
            1.0,
            f64::EPSILON,
            "in_conn_compound",
        )
        .await
    }

    /// Calculate complex incoming connectivities
    pub async fn calculate_complex_connectivity(
        db: &DbConn,
        relation_name: &str,
        to_node: &str,
        weight: f64,
        epsilon: f64,
        col_name: &str,
    ) -> Result<(), DbErr> {
        let builder = db.get_database_backend();
        let node_table = &format_node_table_name(to_node);
        let edge_table = &format_edge_table_name(relation_name);

        let mut node_stmt = sea_query::Query::select();
        node_stmt
            .column(NodeIden::Name)
            .from(Alias::new(node_table))
            .and_where(Expr::col(Alias::new(&format!("{}_in_conn", relation_name))).gt(0));
        let nodes = Node::find_by_statement(builder.build(&node_stmt))
            .all(db)
            .await?;
        let num_nodes = nodes.len();

        let node_to_parents = {
            (Link::find_by_statement(
                builder.build(
                    sea_query::Query::select()
                        .columns([EdgeIden::FromNode, EdgeIden::ToNode])
                        .from(Alias::new(edge_table)),
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

        if map_id_to_ancestors.is_empty() {
            return Ok(());
        }

        // map_id_to_ancestors is ready; the sizes of the sets in its values are the compound in_conn
        let cols = [
            NodeIden::Name.into_iden(),
            Alias::new(&format!("{}_{}", relation_name, col_name)).into_iden(),
        ];
        let mut stmt = Query::insert();
        stmt.into_table(Alias::new(node_table))
            .columns(cols.clone())
            .on_conflict(
                OnConflict::column(NodeIden::Name)
                    .update_columns(cols.clone())
                    .to_owned(),
            );

        for (name, ancestors) in map_id_to_ancestors.into_iter() {
            let in_conn_complex = ancestors
                .into_iter()
                .fold(0.0, |conn, ancestor| conn + ancestor.weight);
            stmt.values_panic([name.into(), in_conn_complex.into()]);
        }

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }
}
