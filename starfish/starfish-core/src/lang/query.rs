use serde::{Deserialize, Serialize};

use crate::query::{QueryResultEdge, QueryResultNode};

use super::ConnectivityTypeJson;

/// Structure of a query request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryJson {
    /// Result is a vector of nodes
    Vector(QueryVectorJson),
    /// Result is a graph of nodes and edges
    Graph(QueryGraphJson),
}

/// Structure of a query request to query a vector, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryVectorJson {
    /// Name of entity
    pub of: String,
    /// Constraints for the query
    pub constraints: Vec<QueryVectorConstraintJson>,
}

/// Structure of a query request to query a graph, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryGraphJson {
    /// Name of entity
    pub of: String,
    /// Constraints for the query
    pub constraints: Vec<QueryGraphConstraintJson>,
}

/// Structure of a common constraint used in a query request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryCommonConstraint {
    /// Sort by a key
    SortBy(QueryConstraintSortByJson),
    /// Limit the number of queried nodes
    Limit(u64),
}

/// Exclusive Structure of a vector constraint used in a query request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryVectorConstraint {
    // Empty
}

/// Structure of a vector constraint used in a query request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QueryVectorConstraintJson {
    /// Common constraint
    Common(QueryCommonConstraint),
    /// Exclusive constraint
    Exclusive(QueryVectorConstraint),
}

/// Exclusive Structure of a graph constraint used in a query request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryGraphConstraint {
    /// Specify edges in which relation to include
    Edge {
        /// Name of relation
        of: String,
        /// Customize traversal
        #[serde(default)]
        traversal: QueryConstraintTraversalJson,
    },
    /// Specify what nodes to use as root nodes
    RootNodes(Vec<String>),
    /// Limit on recursion in graph construction
    Limit(QueryGraphConstraintLimitJson),
}

/// All Structure of a graph constraint used in a query request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QueryGraphConstraintJson {
    /// Common constraint
    Common(QueryCommonConstraint),
    /// Exclusive constraint
    Exclusive(QueryGraphConstraint),
}

/// Structure of a 'sortBy' constraint used in a query request, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConstraintSortByJson {
    /// Key to sort with
    pub key: QueryConstraintSortByKeyJson,
    /// Order of sorting
    #[serde(default)]
    pub desc: bool,
}

/// Key used of a 'sortBy' constraint used in a query request, deserialized as enum from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryConstraintSortByKeyJson {
    /// Sort by connectivity
    Connectivity {
        /// Name of relation to calculate connectivity
        of: String,
        /// Type of connectivity to sort by
        #[serde(default)]
        r#type: ConnectivityTypeJson,
    },
}

/// Structure of a 'limit' constraint used in a query request used to query a graph, deserialized as enum from json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryGraphConstraintLimitJson {
    /// Recurse to a certain depth, 0 means root only.
    /// A `null` value means there is no limit
    Depth(Option<u32>),
    /// Include up to this number of nodes in each batch
    /// A `null` value means there is no limit
    BatchSize(Option<usize>),
}

/// Structure of a traversal method used in a query request, deserialized as struct from json
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryConstraintTraversalJson {
    /// Reverse the direction of edges in traversal
    pub reverse_direction: bool,
}

/// Structure of the result of a query request, to be serialized as json from struct
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QueryResultJson {
    /// A queried vector
    Vector(Vec<QueryResultNode>),
    /// A queried graph
    Graph {
        /// Queried nodes in the graph
        nodes: Vec<QueryResultNode>,
        /// Queried edges in the graph; Must use nodes in `nodes`
        edges: Vec<QueryResultEdge>,
    },
}
