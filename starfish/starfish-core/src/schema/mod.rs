//! Schema management facilities

mod entity;
mod relation;

pub use entity::*;
pub use relation::*;
use sea_orm::{DbConn, DbErr};

use crate::core::lang::SchemaJson;

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
        
        for entity_json in schema_json.define.entities {
            Self::create_entity(db, entity_json).await?;
        }

        for relation_json in schema_json.define.relations {
            Self::create_relation(db, relation_json).await?;
        }

        Ok(())
    }
}