use serde::{Deserialize, Serialize};

use super::{EntityJson, RelationJson};

/// Structure of a schema request, deserialized as struct from json
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
/// Structure of vectors of entities and relations, deserialized as struct from json
pub struct SchemaDefineJson {
    /// Entities schema definition
    pub entities: Vec<EntityJson>,
    /// Relations schema definition
    pub relations: Vec<RelationJson>,
}
