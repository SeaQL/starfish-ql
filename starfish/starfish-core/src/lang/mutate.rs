use std::collections::HashMap;

use sea_orm::JsonValue;
use serde::{Deserialize, Serialize};

use super::{EdgeJsonBatch, NodeJsonBatch};

/// Metadata of a mutate request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum MutateJson {
    /// Insert new data; Use option "upsert" to allow insert-or-update
    insert(MutateInsertJson),
    /// Update selected data
    update(MutateUpdateJson),
    /// Delete selected data
    delete(MutateDeleteJson),
}

/// Metadata of a mutate insert request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum MutateInsertJson {
    /// Insert nodes
    node(NodeJsonBatch),
    /// Insert edges
    edge(EdgeJsonBatch),
}

/// Metadata of a mutate update request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum MutateUpdateJson {
    /// Update nodes
    node {
        /// Selector to select nodes for updating
        selector: MutateNodeSelectorJson,
        /// Specify how to update the selected nodes
        content: HashMap<String, JsonValue>,
    },
    /// Update edges
    edge {
        /// Selector to select edges for updating
        selector: MutateEdgeSelectorJson,
        /// Specify how to update the selected edges
        content: MutateEdgeContentJson,
    },
}

/// Metadata of a mutate delete request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum MutateDeleteJson {
    /// Delete nodes
    node(MutateNodeSelectorJson),
    /// Delete edges
    edge(MutateEdgeSelectorJson),
}

/// Metadata of a node selector of a mutate update/delete request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct MutateNodeSelectorJson {
    /// Name of entity this node belongs to
    pub of: String,
    /// Name of this node
    pub name: Option<String>,
    /// Additional attributes of node
    #[serde(default)]
    pub attributes: HashMap<String, JsonValue>,
}

/// Metadata of an edge selector of a mutate update/delete request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct MutateEdgeSelectorJson {
    /// Name of relation this edge belongs to
    pub of: String,
    /// Specify what edges to look for
    #[serde(flatten)]
    pub edge_content: MutateEdgeContentJson,
}

/// Metadata of the update content of a mutate update request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct MutateEdgeContentJson {
    /// Name of related node (from side, if any)
    pub from_node: Option<String>,
    /// Name of related node (to side, if any)
    pub to_node: Option<String>,
}
