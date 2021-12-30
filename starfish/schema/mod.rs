//! Schema management facilities

mod entity;
mod relation;

pub use entity::*;
pub use relation::*;

/// Unit struct for defining new entity and relation
#[derive(Debug)]
pub struct Schema;

/// Prefix the name of node table
pub fn format_node_table_name<T>(name: T) -> String
where
    T: ToString,
{
    format!("node_{}", name.to_string())
}
