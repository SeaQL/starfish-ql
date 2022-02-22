const { constructUrl } = require("./url");
const { postRequest } = require("./util");

const insertNode = (entity) => async (name, attributes = {}) => {
    await postRequest(
        constructUrl("mutate"),
        {
            insert: {
                node: {
                    of: entity,
                    nodes: [
                        {
                            name,
                            attributes
                        }
                    ]
                }
            }
        }
    )
};
const insertCrateNode = insertNode("crate");

const createNode = (name, attributes = {}) => {
    return { name, attributes };
};

const insertCrateNodesBatch = async (nodes) => {
    await postRequest(
        constructUrl("mutate"),
        {
            insert: {
                node: {
                    of: "crate",
                    nodes
                }
            }
        }
    )
};

const insertEdge = (relation) => async (fromNode, toNode) => {
    await postRequest(
        constructUrl("mutate"),
        {
            insert: {
                edge: {
                    of: relation,
                    edges: [
                        {
                            from_node: fromNode,
                            to_node: toNode
                        }
                    ]
                }
            }
        }
    )
};
const insertDependsEdge = insertEdge("depends");

const createEdge = (fromNode, toNode) => {
    return { from_node: fromNode, to_node: toNode };
};

const insertDependsEdgesBatch = async (edges) => {
    await postRequest(
        constructUrl("mutate"),
        {
            insert: {
                edge: {
                    of: "depends",
                    edges
                }
            }
        }
    )
};

module.exports = {
    insertCrateNode,
    createNode,
    insertCrateNodesBatch,
    insertDependsEdge,
    createEdge,
    insertDependsEdgesBatch,
};