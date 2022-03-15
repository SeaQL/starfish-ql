use serde::{Deserialize, Serialize};

use super::{EntityJson, RelationJson};

/// Metadata of a schema request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SchemaJson {
    /// What this defines
    Define(SchemaDefineJson),
    /// Signal a reset of schema
    Reset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Metadata of vectors of entities and relations, deserialized as struct from json
pub struct SchemaDefineJson {
    /// Entities metadata
    pub entities: Vec<EntityJson>,
    /// Relations metadata
    pub relations: Vec<RelationJson>,
}
