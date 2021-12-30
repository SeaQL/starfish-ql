use super::Mutate;
use crate::{
    core::entities::{entity, entity_attribute},
    schema::{format_node_attribute_name, format_node_table_name},
};
use sea_orm::{ColumnTrait, ConnectionTrait, DbConn, DbErr, EntityTrait, QueryFilter, Value};
use sea_query::{Alias, Expr, Query};
use std::collections::HashMap;

/// Metadata of a node, deserialized as struct from json
#[derive(Debug)]
pub struct NodeJson {
    /// Name of entity this node belongs to
    pub of: String,
    /// Name of node
    pub name: String,
    /// Additional attributes
    pub attributes: HashMap<String, Value>,
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

        let mut cols = vec![Alias::new("name")];
        let mut vals = vec![node_json.name.into()];

        let attributes = &vec[0].1;
        for attribute in attributes.iter() {
            let name = &attribute.name;
            if let Some(val) = node_json.attributes.get(name) {
                cols.push(Alias::new(&format_node_attribute_name(name)));
                vals.push(val.clone());
            }
        }

        let mut stmt = Query::insert();
        stmt.into_table(Alias::new(&format_node_table_name(node_json.of)))
            .columns(cols)
            .values_panic(vals);

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
