//! Abstract Syntax Tree for the query language, constructed using JSON

/// Structure of a schema request
pub mod schema;

/// Structure of a mutate request
pub mod mutate;

/// Structure of a query request
pub mod query;

/// Reusable column identifiers in requests
pub mod iden;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::schema::{format_edge_table_name, format_node_attribute_name, format_node_table_name};

use super::entities::entity_attribute::Datatype;

/// Structure of entity, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityJson {
    /// Name of entity
    pub name: String,
    /// Additional attributes
    pub attributes: Vec<EntityAttrJson>,
}

/// Structure of entity attribute, deserialized as struct from json
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

/// Structure of relation, deserialized as struct from json
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

/// Structure of a edge, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EdgeJson {
    /// Name of relation
    pub name: String,
    /// Name of related node (from side)
    pub from_node: String,
    /// Name of related node (to side)
    pub to_node: String,
}

/// Structure of a edge, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClearEdgeJson {
    /// Name of relation
    pub name: String,
    /// Name of node
    pub node: String,
}

/// Structure of a node, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeJson {
    /// Name of entity this node belongs to
    pub of: String,
    /// Name of node
    pub name: String,
    /// Additional attributes
    pub attributes: HashMap<String, JsonValue>,
}

/// Structure of a node in batch, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Node {
    /// Name of node
    pub name: String,
    /// Additional attributes
    pub attributes: HashMap<String, JsonValue>,
}

impl Node {
    /// Construct a new Node with no attributes
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            attributes: Default::default(),
        }
    }

    /// Construct a vector of new Nodes with no attributes
    pub fn new_vec<S>(names: Vec<S>) -> Vec<Self>
    where
        S: Into<String>,
    {
        names.into_iter().map(Self::new).collect()
    }
}

/// Structure of nodes in batch, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeJsonBatch {
    /// Name of entity this node belongs to
    pub of: String,
    /// Vector of nodes
    pub nodes: Vec<Node>,
}

/// Structure of a edge in batch, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Edge {
    /// Name of related node (from side)
    pub from_node: String,
    /// Name of related node (to side)
    pub to_node: String,
}

impl Edge {
    /// Construct a new Edge
    pub fn new<SFrom, STo>(from: SFrom, to: STo) -> Self
    where
        SFrom: Into<String>,
        STo: Into<String>,
    {
        Self {
            from_node: from.into(),
            to_node: to.into(),
        }
    }

    /// Construct a vector of new Edges
    pub fn new_vec<SFrom, STo>(pairs: Vec<(SFrom, STo)>) -> Vec<Self>
    where
        SFrom: Into<String>,
        STo: Into<String>,
    {
        pairs
            .into_iter()
            .map(|(from, to)| Self::new(from, to))
            .collect()
    }
}

/// Structure of edges in batch, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EdgeJsonBatch {
    /// Name of relation
    pub of: String,
    /// Vector of edges
    pub edges: Vec<Edge>,
}

/// Structure of the connectivity in a 'sortBy' constraint used in a query request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectivityTypeJson {
    /// Simple in-connectivity
    Simple,
    /// Compound in-connectivity
    Compound,
    /// Complex in-connectivity with decay factor 0.3
    Complex03,
    /// Complex in-connectivity with decay factor 0.5
    Complex05,
    /// Complex in-connectivity with decay factor 0.7
    Complex07,
    /// out-connectivity
    Out,
}

impl Default for ConnectivityTypeJson {
    fn default() -> Self {
        Self::Simple
    }
}

impl ConnectivityTypeJson {
    /// Convert self to the corresponding column name in the entity table
    pub fn to_column_name<S: Into<String>>(self, relation_name: S) -> String {
        format!(
            "{}_{}",
            relation_name.into(),
            match self {
                Self::Simple => "in_conn",
                Self::Compound => "in_conn_compound",
                Self::Complex03 => "in_conn_complex03",
                Self::Complex05 => "in_conn_complex05",
                Self::Complex07 => "in_conn_complex07",
                Self::Out => "out_conn",
            }
        )
    }
}
