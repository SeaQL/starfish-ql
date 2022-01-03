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
};;
const insertCrateNode = insertNode("crate");

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

module.exports = {
    insertCrateNode,
    insertDependsEdge,
};