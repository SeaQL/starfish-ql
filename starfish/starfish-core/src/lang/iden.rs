use sea_orm::DynIden;
use sea_query::{Iden, Alias, IntoIden};

/// Reusable column identifiers for edges
#[derive(Debug, Iden)]
pub enum EdgeIden {
    /// The `from_node` column of an edge
    #[iden = "from_node"]
    FromNode,
    /// The `to_node` column of an edge
    #[iden = "to_node"]
    ToNode,
}

/// Reusable column identifiers for nodes
#[derive(Debug, Iden)]
pub enum NodeIden {
    /// The `name` column of a node
    #[iden = "name"]
    Name,
}

