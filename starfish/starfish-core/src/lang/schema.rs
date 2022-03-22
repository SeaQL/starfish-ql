use serde::{Deserialize, Serialize};

use super::{EntityJson, RelationJson};

/// Metadata of a schema request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaJson {
    /// If true, define all schema from scratch. Defaults to be false (append mode).
    #[serde(default)]
    pub reset: bool,
    /// What this defines
    #[serde(default)]
    pub define: SchemaDefineJson,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Metadata of vectors of entities and relations, deserialized as struct from json
pub struct SchemaDefineJson {
    /// Entities metadata
    pub entities: Vec<EntityJson>,
    /// Relations metadata
    pub relations: Vec<RelationJson>,
}
