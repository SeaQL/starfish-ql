use super::Mutate;
use crate::{
    core::entities::{
        entity,
        entity_attribute::{self, Datatype},
    },
    schema::{format_node_attribute_name, format_node_table_name},
};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbConn, DbErr, EntityTrait, FromQueryResult, QueryFilter,
};
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

#[derive(Debug, FromQueryResult)]
struct Node {
    id: i32,
}

impl Mutate {
    /// Insert node
    pub async fn insert_node(db: &DbConn, node_json: NodeJson) -> Result<(), DbErr> {
        let vec = entity::Entity::find()
            .find_with_related(entity_attribute::Entity)
            .filter(entity::Column::Name.eq(node_json.of.as_str()))
            .all(db)
            .await?;

        if vec.is_empty() {
            return Err(DbErr::Custom(format!(
                "Entity of name '{}' could not be found",
                node_json.of
            )));
        }

        let builder = db.get_database_backend();
        let node_table = format_node_table_name(node_json.of);
        let mut cols = vec![Alias::new("name")];
        let mut vals = vec![node_json.name.as_str().into()];

        let attributes = &vec[0].1;
        for attribute in attributes.iter() {
            let name = &attribute.name;
            if let Some(val) = node_json.attributes.get(name) {
                let val = match attribute.datatype {
                    Datatype::Int => val.as_i64().into(),
                    Datatype::String => val.as_str().into(),
                };
                cols.push(Alias::new(&format_node_attribute_name(name)));
                vals.push(val);
            }
        }

        let mut stmt = Query::select();
        stmt.from(Alias::new(&node_table))
            .expr(Expr::col(Alias::new("id")))
            .and_where(Expr::col(Alias::new("name")).eq(node_json.name));

        let node = Node::find_by_statement(builder.build(&stmt))
            .one(db)
            .await?;

        let stmt = if let Some(node) = node {
            let mut stmt = Query::update();
            stmt.table(Alias::new(&node_table))
                .values(cols.into_iter().zip(vals.into_iter()))
                .and_where(Expr::col(Alias::new("id")).eq(node.id));
            builder.build(&stmt)
        } else {
            let mut stmt = Query::insert();
            stmt.into_table(Alias::new(&node_table))
                .columns(cols)
                .values_panic(vals);
            builder.build(&stmt)
        };

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
}
