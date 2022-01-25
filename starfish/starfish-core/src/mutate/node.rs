use std::collections::HashMap;

use super::Mutate;
use crate::{
    entities::{
        entity,
        entity_attribute::{self, Datatype},
    },
    lang::{Node, NodeJson, NodeJsonBatch, MutateNodeSelectorJson},
    schema::{format_node_attribute_name, format_node_table_name},
};
use sea_orm::{ColumnTrait, ConnectionTrait, DbConn, DbErr, DeriveIden, EntityTrait, QueryFilter, JsonValue, Value};
use sea_query::{Alias, Expr, Query, Cond};

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
            true,
        )
        .await
    }

    /// Insert node in batch
    pub async fn insert_node_batch(
        db: &DbConn,
        node_json_batch: NodeJsonBatch,
        upsert: bool,
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
            .columns(cols.clone());

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
        let mut stmt = builder.build(&stmt);
        if upsert {
            let update_vals = cols
                .into_iter()
                .map(|col| {
                    let col = col.to_string();
                    format!("{0} = VALUES({0})", col)
                })
                .collect::<Vec<_>>()
                .join(", ");

            stmt.sql = format!("{} ON DUPLICATE KEY UPDATE {}", stmt.sql, update_vals);
        }
        db.execute(stmt).await?;

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

    /// Update node
    pub async fn update_node(db: &DbConn, selector: MutateNodeSelectorJson, content: HashMap<String, JsonValue>) -> Result<(), DbErr> {
        let set_values: Vec<(Alias, Value)> = content.into_iter().map(|(k, v)| {
            (Alias::new(&format_node_attribute_name(k)), v.into())
        }).collect();

        let condition = selector.attributes.into_iter().fold(Cond::all(), |cond, (k, v)| {
            cond.add(Expr::col(Alias::new(&format_node_attribute_name(k))).eq(v))
        });

        let mut stmt = Query::update();

        stmt.table(Alias::new(&format_node_table_name(selector.of)))
            .values(set_values)
            .cond_where(
                if let Some(name) = selector.name {
                    condition.add(Expr::col(Alias::new("name")).eq(name))
                } else {
                    condition
                }
            );

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;
        
        Ok(())
    }
}
