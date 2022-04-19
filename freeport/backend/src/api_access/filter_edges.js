// Assumes both `edges` and `nodes` are arrays.
// Their interfaces can be found in 'insert.js'.
function filterEdges(edges, nodes) {
    const nodeSet = new Set(nodes.map((node) => node.name));

    const bothNodesExist = (({from_node, to_node}) => {
        return nodeSet.has(from_node) && nodeSet.has(to_node);
    });

    return edges.filter(bothNodesExist);
}

module.exports.default = filterEdges;
