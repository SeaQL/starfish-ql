//! Define entity schema

use super::{format_node_attribute_name, format_node_table_name, Schema};
use crate::core::entities::{
    entity,
    entity_attribute::{self, Datatype},
};
use sea_orm::{ActiveModelTrait, ConnectionTrait, DbConn, DbErr, DeriveIden, Set};
use sea_query::{Alias, ColumnDef, Index, Table};
use serde::{Deserialize, Serialize};

/// Metadata of entity, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityJson {
    /// Name of entity
    pub name: String,
    /// Additional attributes
    pub attributes: Vec<EntityAttrJson>,
}

/// Metadata of entity attribute, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        format_node_attribute_name(&self.name)
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
                name: Set(attribute.name),
                datatype: Set(attribute.datatype),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }

        Self::create_node_table(db, entity_json_bak).await
    }

    async fn create_node_table(db: &DbConn, entity_json: EntityJson) -> Result<(), DbErr> {
        let table = Alias::new(entity_json.get_table_name().as_str());
        let mut stmt = Table::create();
        stmt.table(table.clone())
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
            )
            .col(
                ColumnDef::new(Alias::new("in_conn"))
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(Alias::new("in_conn_compound"))
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(Alias::new("in_conn_complex03"))
                    .double()
                    .not_null()
                    .default(0.0),
            )
            .col(
                ColumnDef::new(Alias::new("in_conn_complex05"))
                    .double()
                    .not_null()
                    .default(0.0),
            )
            .col(
                ColumnDef::new(Alias::new("in_conn_complex07"))
                    .double()
                    .not_null()
                    .default(0.0),
            )
            .col(
                ColumnDef::new(Alias::new("out_conn"))
                    .integer()
                    .not_null()
                    .default(0),
            )
            .index(
                Index::create()
                    .name(&format!("idx-{}-{}", table.to_string(), "name"))
                    .table(table.clone())
                    .col(Alias::new("name")),
            )
            .index(
                Index::create()
                    .name(&format!("idx-{}-{}", table.to_string(), "in_conn"))
                    .table(table.clone())
                    .col(Alias::new("in_conn")),
            )
            .index(
                Index::create()
                    .name(&format!("idx-{}-{}", table.to_string(), "in_conn_compound"))
                    .table(table.clone())
                    .col(Alias::new("in_conn_compound")),
            )
            .index(
                Index::create()
                    .name(&format!("idx-{}-{}", table.to_string(), "in_conn_complex03"))
                    .table(table.clone())
                    .col(Alias::new("in_conn_complex03")),
            )
            .index(
                Index::create()
                    .name(&format!("idx-{}-{}", table.to_string(), "in_conn_complex05"))
                    .table(table.clone())
                    .col(Alias::new("in_conn_complex05")),
            )
            .index(
                Index::create()
                    .name(&format!("idx-{}-{}", table.to_string(), "in_conn_complex07"))
                    .table(table.clone())
                    .col(Alias::new("in_conn_complex07")),
            )
            .index(
                Index::create()
                    .name(&format!("idx-{}-{}", table.to_string(), "out_conn"))
                    .table(table.clone())
                    .col(Alias::new("out_conn")),
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
