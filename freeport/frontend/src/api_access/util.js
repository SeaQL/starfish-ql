const axios = require("axios").default;

export const postRequest = async (url, data, config = {}) => {
    return await axios.post(url, JSON.stringify(data), config);
}

export const sleep = (ms) => {
    return new Promise((resolve) => setTimeout(resolve, ms));
};

export const formatGraphData = (graphData) => {
    return {
        nodes: graphData.nodes.map((node) => {
            return {
                id: node.name,
                weight: node.weight,
            }
        }),
        links: graphData.edges.map((edge) => {
            return {
                source: edge.fromNode,
                target: edge.toNode,
            }
        })
    };
};