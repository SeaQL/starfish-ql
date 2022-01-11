export const highlightConnectedNodesAndLinks = (subjectId, nodes, links) => {
    const connectedNodeIds = new Set();

    links.attr("opacity", (d) => {
        let opacity = 0.1;

        if (subjectId === d.source.id) {
            opacity = 1;
            connectedNodeIds.add(d.target.id);
        } else if (subjectId === d.target.id) {
            opacity = 1;
            connectedNodeIds.add(d.source.id);
        }

        return opacity;
    });

    // Traverse nodes, set opacity if id is in the store
    nodes.attr("opacity", (d) => {
        return connectedNodeIds.has(d.id) || d.id === subjectId ? 1 : 0.25;
    });
};

export const resetAllHighlight = (nodes, links) => {
    links.attr("opacity", 1);
    nodes.attr("opacity", 1);
};