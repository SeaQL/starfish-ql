use std::collections::HashMap;

use sea_orm::JsonValue;
use serde::{Serialize, Deserialize};

use super::ConnectivityTypeJson;

/// Metadata of a query request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum QueryJson {
    /// Result is a vector of nodes
    vector(QueryVectorJson),
    /// Result is a graph of nodes and edges
    graph(QueryGraphJson),
}

/// Metadata of a query request to query a vector, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryVectorJson {
    /// Constraints for the query
    pub constraints: Vec<QueryConstraintJson>,
}

/// Metadata of a query request to query a graph, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryGraphJson {
    /// Constraints for the query
    pub constraints: Vec<QueryConstraintJson>,
}

/// Metadata of a constraint used in a query request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum QueryConstraintJson {
    /// Sort by a key
    sortBy(QueryConstraintSortByJson),
    /// Limit the number of queried nodes
    limit(QueryConstraintLimitJson),
    /// Specify edges in which relation to include, only valid when querying a graph
    edge {
        /// Name of relation
        of: String,
        /// Customize traversal
        #[serde(default)]
        traversal: QueryConstraintTraversalJson,
    },
    /// Specify what nodes to use as root nodes, only valid when querying a graph
    rootNodes(Vec<HashMap<String, JsonValue>>),
}

/// Metadata of a 'sortBy' constraint used in a query request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum QueryConstraintSortByJson {
    /// Sort by connectivity
    connectivity {
        /// Name of relation to calculate connectivity
        of: String,
        /// Type of connectivity to sort by
        #[serde(default)]
        r#type: ConnectivityTypeJson,
    }
}

/// Metadata of a 'limit' constraint used in a query request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum QueryConstraintLimitJson {
    /// Limit by a range
    range(QueryConstraintLimitRangeJson),
    /// Recurse to a certain depth
    depth {
        /// Recurse to this depth, 0 means root only
        to: usize,
    },
}

/// Metadata of a range in a 'limit' constraint used in a query request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum QueryConstraintLimitRangeJson {
    /// Get the top n nodes
    top(usize),
    /// Get the bottom n nodes
    bottom(usize),
}

/// Metadata of a traversal method used in a query request, deserialized as struct from json
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryConstraintTraversalJson {
    /// Reverse the direction of edges in traversal
    pub reverse_direction: bool,
}
