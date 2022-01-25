//! Abstract Syntax Tree for the query language

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::schema::{format_edge_table_name, format_node_attribute_name, format_node_table_name};

use super::entities::entity_attribute::Datatype;

/// Metadata of a schema request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaJson {
    /// What this defines
    pub define: SchemaDefineJson,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Metadata of vectors of entities and relations, deserialized as struct from json
pub struct SchemaDefineJson {
    /// Entities metadata
    pub entities: Vec<EntityJson>,
    /// Relations metadata
    pub relations: Vec<RelationJson>,
}

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
    }
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
    /// Attributes of node, primary and additional
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

/// Metadata of entity, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityJson {
    /// Name of entity
    pub name: String,
    /// Additional attributes
    pub attributes: Vec<EntityAttrJson>,
}

/// Metadata of entity attribute, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityAttrJson {
    /// Name of attribute
    pub name: String,
    /// Datatype, to determine how to store the value in database
    pub datatype: Datatype,
}

impl EntityJson {
    /// Prefix the name of node table
    pub fn get_table_name(&self) -> String {
        format_node_table_name(&self.name)
    }
}

impl EntityAttrJson {
    /// Prefix the column name of entity attribute
    pub fn get_column_name(&self) -> String {
        format_node_attribute_name(&self.name)
    }
}

/// Metadata of relation, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationJson {
    /// Name of relation
    pub name: String,
    /// Name of related entity (from side)
    pub from_entity: String,
    /// Name of related entity (to side)
    pub to_entity: String,
    /// Directed relation
    pub directed: bool,
}

impl RelationJson {
    /// Prefix the name of relation table
    pub fn get_table_name(&self) -> String {
        format_edge_table_name(&self.name)
    }
}

/// Metadata of a edge, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EdgeJson {
    /// Name of relation
    pub name: String,
    /// Name of related node (from side)
    pub from_node: String,
    /// Name of related node (to side)
    pub to_node: String,
}

/// Metadata of a edge, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClearEdgeJson {
    /// Name of relation
    pub name: String,
    /// Name of node
    pub node: String,
}

/// Metadata of a node, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeJson {
    /// Name of entity this node belongs to
    pub of: String,
    /// Name of node
    pub name: String,
    /// Additional attributes
    pub attributes: HashMap<String, JsonValue>,
}

/// Metadata of a node in batch, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Node {
    /// Name of node
    pub name: String,
    /// Additional attributes
    pub attributes: HashMap<String, JsonValue>,
}

/// Metadata of nodes in batch, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeJsonBatch {
    /// Name of entity this node belongs to
    pub of: String,
    /// Vector of nodes
    pub nodes: Vec<Node>,
}

/// Metadata of a edge in batch, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Edge {
    /// Name of related node (from side)
    pub from_node: String,
    /// Name of related node (to side)
    pub to_node: String,
}

/// Metadata of edges in batch, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EdgeJsonBatch {
    /// Name of relation
    pub of: String,
    /// Vector of edges
    pub edges: Vec<Edge>,
}
