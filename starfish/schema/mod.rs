//! Schema management facilities

mod entity;
mod relation;

pub use entity::*;
pub use relation::*;

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
