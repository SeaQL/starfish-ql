const { constructUrl } = require("./url");
const { postRequest } = require("./util");

const insertNode = (entity) => async (name, attributes = {}) => {
    await postRequest(
        constructUrl("mutate/insert-node"),
        {
            of: entity,
            name,
            attributes,
        }
    )
};
const insertCrateNode = insertNode("crate");

const createNode = (entity) => (name, attributes = {}) => {
    return { of: entity, name, attributes };
};
const createCrateNode = createNode("crate");

const insertNodesBatch = async (nodes) => {
    await postRequest(
        constructUrl("mutate/insert-node-batch"),
        nodes
    )
};

const insertEdge = (relation) => async (fromNode, toNode) => {
    await postRequest(
        constructUrl("mutate/insert-edge"),
        {
            name: relation,
            from_node: fromNode,
            to_node: toNode
        }
    )
};
const insertDependsEdge = insertEdge("depends");

const createEdge = (relation) => (fromNode, toNode) => {
    return { name: relation, from_node: fromNode, to_node: toNode };
};
const createDependsEdge = createEdge("depends");

const insertEdgesBatch = async (edges) => {
    await postRequest(
        constructUrl("mutate/insert-edge-batch"),
        edges
    )
};

module.exports = {
    insertCrateNode,
    createCrateNode,
    insertNodesBatch,
    insertDependsEdge,
    createDependsEdge,
    insertEdgesBatch,
};