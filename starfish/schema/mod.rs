//! Schema management facilities

mod entity;
mod relation;

pub use entity::*;
pub use relation::*;

#[derive(Debug)]
pub struct Schema;

pub fn format_node_table_name<T>(name: T) -> String
where
    T: ToString,
{
    format!("node_{}", name.to_string())
}
