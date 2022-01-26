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
use sea_orm::{ColumnTrait, ConnectionTrait, DbConn, DbErr, DeriveIden, EntityTrait, QueryFilter, JsonValue, Value, FromQueryResult, JoinType};
use sea_query::{Alias, Expr, Query, Cond};

#[derive(Debug, Clone, FromQueryResult)]
struct AttributeMeta {
    name: String,
    datatype: Datatype,
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
                let val = attribute.datatype.value_with_datatype(node_json.attributes.get(name));
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

    /// Delete node with a selector
    pub async fn delete_node_with_selector(db: &DbConn, selector: MutateNodeSelectorJson) -> Result<(), DbErr> {
        let condition = selector.attributes.into_iter().fold(Cond::all(), |cond, (k, v)| {
            cond.add(Expr::col(Alias::new(&format_node_attribute_name(k))).eq(v.as_str().unwrap()))
        });

        let stmt = Query::delete()
            .from_table(Alias::new(&format_node_table_name(selector.of)))
            .cond_where(
                if let Some(name) = selector.name {
                    condition.add(Expr::col(Alias::new("name")).eq(name))
                } else {
                    condition
                }
            )
            .to_owned();

        let builder = db.get_database_backend();

        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }

    /// Update node
    pub async fn update_node_attributes(db: &DbConn, selector: MutateNodeSelectorJson, content: HashMap<String, JsonValue>) -> Result<(), DbErr> {
        let builder = db.get_database_backend();

        let entity_attribute_alias = Alias::new("entity_attribute");
        let entity_alias = Alias::new("entity");

        let attr_stmt = Query::select()
            .column((entity_attribute_alias.clone(), Alias::new("name")))
            .column(Alias::new("datatype"))
            .from(entity_alias.clone())
            .join(
                JoinType::Join,
                entity_attribute_alias.clone(),
                Expr::tbl(entity_alias.clone(), Alias::new("id"))
                    .equals(entity_attribute_alias, Alias::new("entity_id"))
            )
            .and_where(Expr::col((entity_alias, Alias::new("name"))).eq(selector.of.clone()))
            .to_owned();
        let attributes = AttributeMeta::find_by_statement(builder.build(&attr_stmt))
            .all(db)
            .await?
            .into_iter()
            .fold(HashMap::new(), |mut map, attribute_meta| {
                map.insert(attribute_meta.name, attribute_meta.datatype);
                map
            });

        let set_values = content.into_iter().map(|(k, v)| {
            if let Some(dtype) = attributes.get(&k) {
                Ok((Alias::new(&format_node_attribute_name(k)), dtype.value_with_datatype(Some(&v))))
            } else {
                Err(DbErr::Custom(format!("Datatype of attribute \"{}\" is not defined.", k)))
            }
        }).collect::<Result<Vec<(Alias, Value)>, DbErr>>()?;

        let condition = selector.attributes.into_iter().fold(Cond::all(), |cond, (k, v)| {
            cond.add(Expr::col(Alias::new(&format_node_attribute_name(k))).eq(v.as_str().unwrap()))
        });

        let update_stmt = Query::update()
            .table(Alias::new(&format_node_table_name(selector.of)))
            .values(set_values)
            .cond_where(
                if let Some(name) = selector.name {
                    condition.add(Expr::col(Alias::new("name")).eq(name))
                } else {
                    condition
                }
            )
            .to_owned();

        db.execute(builder.build(&update_stmt)).await?;
        
        Ok(())
    }
}
