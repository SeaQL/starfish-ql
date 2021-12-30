//! Define entity schema

use super::{format_node_table_name, Schema};
use crate::core::entities::{
    entity,
    entity_attribute::{self, Datatype},
};
use sea_orm::{ActiveModelTrait, ConnectionTrait, DbConn, DbErr, Set};
use sea_query::{Alias, ColumnDef, Table};

/// Metadata of entity, deserialized as struct from json
#[derive(Debug, Clone)]
pub struct EntityJson {
    /// Name of entity
    pub name: String,
    /// Additional attributes
    pub attributes: Vec<EntityAttrJson>,
}

/// Metadata of entity attribute, deserialized as struct from json
#[derive(Debug, Clone)]
pub struct EntityAttrJson {
    /// Name of attribute
    pub name: String,
    /// Datatype, to determine how to store the value in database
    pub datatype: Datatype,
}

impl EntityJson {
    /// Prefix the name of node table
    pub fn get_table_name(&self) -> String {
        format_node_table_name(&self.name)
    }
}

impl EntityAttrJson {
    /// Prefix the column name of entity attribute
    pub fn get_column_name(&self) -> String {
        format!("attr_{}", self.name)
    }
}

impl Schema {
    /// Insert entity metadata into database and create a corresponding node table
    pub async fn create_entity(db: &DbConn, entity_json: EntityJson) -> Result<(), DbErr> {
        let entity_json_bak = entity_json.clone();

        let entity = entity::ActiveModel {
            name: Set(entity_json.name),
            ..Default::default()
        }
        .insert(db)
        .await?;

        for attribute in entity_json.attributes.into_iter() {
            entity_attribute::ActiveModel {
                entity_id: Set(entity.id),
                name: Set(attribute.get_column_name()),
                datatype: Set(attribute.datatype),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }

        Self::create_node_table(db, entity_json_bak).await
    }

    async fn create_node_table(db: &DbConn, entity_json: EntityJson) -> Result<(), DbErr> {
        let mut stmt = Table::create();

        stmt.table(Alias::new(entity_json.get_table_name().as_str()))
            .col(
                ColumnDef::new(Alias::new("id"))
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(Alias::new("name"))
                    .string()
                    .not_null()
                    .unique_key(),
            );

        for attribute in entity_json.attributes.into_iter() {
            let mut column_def = ColumnDef::new(Alias::new(attribute.get_column_name().as_str()));
            match attribute.datatype {
                Datatype::Int => column_def.integer(),
                Datatype::String => column_def.string(),
            };
            stmt.col(&mut column_def);
        }

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }
}
