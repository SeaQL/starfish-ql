use super::Mutate;
use crate::{
    core::entities::{
        entity,
        entity_attribute::{self, Datatype},
    },
    schema::{format_node_attribute_name, format_node_table_name},
};
use sea_orm::{ColumnTrait, ConnectionTrait, DbConn, DbErr, EntityTrait, QueryFilter};
use sea_query::{Alias, Expr, Query};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Metadata of a node, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeJson {
    /// Name of entity this node belongs to
    pub of: String,
    /// Name of node
    pub name: String,
    /// Additional attributes
    pub attributes: HashMap<String, JsonValue>,
}

/// Metadata of nodes in batch, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeJsonBatch {
    /// Name of entity this node belongs to
    pub of: String,
    /// Vector of nodes
    pub nodes: Vec<Node>,
}

/// Metadata of a node in batch, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Node {
    /// Name of node
    pub name: String,
    /// Additional attributes
    pub attributes: HashMap<String, JsonValue>,
}

impl Mutate {
    /// Insert node
    pub async fn insert_node(db: &DbConn, node_json: NodeJson) -> Result<(), DbErr> {
        Self::insert_node_batch(
            db,
            NodeJsonBatch {
                of: node_json.of,
                nodes: vec![Node {
                    name: node_json.name,
                    attributes: node_json.attributes,
                }],
            },
        )
        .await
    }

    /// Insert node in batch
    pub async fn insert_node_batch(
        db: &DbConn,
        node_json_batch: NodeJsonBatch,
    ) -> Result<(), DbErr> {
        let vec = entity::Entity::find()
            .find_with_related(entity_attribute::Entity)
            .filter(entity::Column::Name.eq(node_json_batch.of.as_str()))
            .all(db)
            .await?;

        if vec.is_empty() {
            return Err(DbErr::Custom(format!(
                "Entity of name '{}' could not be found",
                node_json_batch.of
            )));
        }

        let mut cols = vec![Alias::new("name")];
        let attributes = &vec[0].1;

        for attribute in attributes.iter() {
            cols.push(Alias::new(&format_node_attribute_name(&attribute.name)));
        }

        let mut stmt = Query::insert();
        stmt.into_table(Alias::new(&format_node_table_name(node_json_batch.of)))
            .columns(cols);

        for node_json in node_json_batch.nodes.into_iter() {
            let mut vals = vec![node_json.name.as_str().into()];
            for attribute in attributes.iter() {
                let name = &attribute.name;
                let val = if let Some(val) = node_json.attributes.get(name) {
                    match attribute.datatype {
                        Datatype::Int => val.as_i64().into(),
                        Datatype::String => val.as_str().into(),
                    }
                } else {
                    match attribute.datatype {
                        Datatype::Int => None::<i64>.into(),
                        Datatype::String => None::<String>.into(),
                    }
                };
                vals.push(val);
            }
            stmt.values_panic(vals);
        }

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }

    /// Delete node
    pub async fn delete_node(db: &DbConn, of: String, node_name: String) -> Result<(), DbErr> {
        entity::Entity::find()
            .filter(entity::Column::Name.eq(of.as_str()))
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom(format!("Entity of name '{}' could not be found", of)))?;

        let mut stmt = Query::delete();
        stmt.from_table(Alias::new(&format_node_table_name(of)))
            .and_where(Expr::col(Alias::new("name")).eq(node_name));

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }
}
