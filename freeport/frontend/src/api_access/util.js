import { TreeElemType } from "../gui/render_tree";

const axios = require("axios").default;

export const postRequest = async (url, data, config = {}) => {
    return await axios.post(url, JSON.stringify(data), config);
}

export const sleep = (ms) => {
    return new Promise((resolve) => setTimeout(resolve, ms));
};

const formatEdges = (edges) => edges.map((edge) => {
    return {
        source: edge.fromNode,
        target: edge.toNode,
    }
});

export const formatGraphData = (graphData) => {
    return {
        nodes: graphData.nodes.map((node) => {
            return {
                id: node.name,
                weight: node.weight,
            }
        }),
        links: formatEdges(graphData.edges)
    };
};

export const formatTreeData = (rootNode, lhsData, rhsData) => {
    const combined = {
        nodes: [ {id: rootNode, type: TreeElemType.Root} ],
        links: [ ...formatEdges(lhsData.edges), ...formatEdges(rhsData.edges)],
    };

    combined.nodes.push(...lhsData.nodes.map((node) => {
        return {
            id: node.name,
            type: TreeElemType.Dependency,
        }
    }));

    combined.nodes.push(...rhsData.nodes.map((node) => {
        return {
            id: node.name,
            type: TreeElemType.Dependent,
        }
    }));

    combined.nodes = combined.nodes.filter((node) => {
        return node.id !== rootNode || node.type === TreeElemType.Root;
    });

    console.log(combined);

    return combined;
};