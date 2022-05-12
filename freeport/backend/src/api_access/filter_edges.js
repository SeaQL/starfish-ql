// Assumes both `edges` and `nodes` are arrays.
// Their interfaces can be found in 'insert.js'.
function filterEdges(edges, nodeSet) {
    const invalidEdges = [];

    const bothNodesExist = (({from_node, to_node}) => {
        return nodeSet.has(from_node) && nodeSet.has(to_node);
    });

    edges = edges.filter((edge) => {
        if (bothNodesExist(edge)) {
            return true;
        } else {
            invalidEdges.push(edge);
            return false;
        }
    });

    return {
        valid: edges,
        invalid: invalidEdges,
    };
}

module.exports.default = filterEdges;
