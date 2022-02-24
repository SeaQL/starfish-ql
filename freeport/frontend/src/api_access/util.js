import { TreeElemType } from "../gui/render_tree";

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

export const formatTreeData = (rootNode, lhsData, rhsData) => {
    const combined = {
        nodes: [ {id: rootNode, type: TreeElemType.Root, depthInv: 0} ],
        links: [],
    };

    combined.nodes.push(...lhsData.nodes.map((node) => {
        return {
            id: node.name,
            type: TreeElemType.Dependency,
            depthInv: 1
        }
    }));

    combined.nodes.push(...rhsData.nodes.map((node) => {
        return {
            id: node.name,
            type: TreeElemType.Dependent,
            depthInv: 1
        }
    }));

    combined.nodes = combined.nodes.filter((node) => {
        return node.id !== rootNode || node.type === TreeElemType.Root;
    });

    // Each link is also associated with a type to indicate which side it is on for coloring
    combined.links.push(...lhsData.edges.map((edge) => {
        return {
            source: edge.fromNode,
            target: edge.toNode,
            type: TreeElemType.Dependency,
        }
    }));
    combined.links.push(...rhsData.edges.map((edge) => {
        return {
            source: edge.fromNode,
            target: edge.toNode,
            type: TreeElemType.Dependent,
        }
    }));

    console.log(combined);

    return combined;
};