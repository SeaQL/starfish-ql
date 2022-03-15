//! Schema management facilities

mod entity;
mod relation;

pub use entity::*;
pub use relation::*;
use sea_orm::{DbConn, DbErr};
use sea_schema::migration::MigratorTrait;

use crate::{lang::schema::SchemaJson, migrator::Migrator};

/// Define new entity and relation
#[derive(Debug)]
pub struct Schema;

/// Prefix the name of node table
pub fn format_node_table_name<T>(name: T) -> String
where
    T: ToString,
{
    format!("node_{}", name.to_string())
}

/// Prefix the name of node attribute column
pub fn format_node_attribute_name<T>(name: T) -> String
where
    T: ToString,
{
    format!("attr_{}", name.to_string())
}

/// Prefix the name of edge table
pub fn format_edge_table_name<T>(name: T) -> String
where
    T: ToString,
{
    format!("edge_{}", name.to_string())
}

impl Schema {
    /// Insert entity/relation metadata into database and create a corresponding node/edge table
    pub async fn define_schema(db: &DbConn, schema_json: SchemaJson) -> Result<(), DbErr> {
        match schema_json {
            SchemaJson::Define(schema_define_json) => {
                for entity_json in schema_define_json.entities {
                    Self::create_entity(db, entity_json).await?;
                }
                for relation_json in schema_define_json.relations {
                    Self::create_relation(db, relation_json).await?;
                }
            },
            SchemaJson::Reset => {
                Migrator::fresh(db).await.map_err(Into::into)?;
            },
        }

        Ok(())
    }
}
