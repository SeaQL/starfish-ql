import { formatTreeData, postRequest } from "./util";
import { constructUrl } from "./url";
import MOCK_TREE from "../data/mock_tree.json";

export const getTree = async (rootNode, limit, depth, weightDecayMode) => {
    const url = constructUrl("query");

    const config = {
        graph: {
            of: "crate",
            constraints: [
                {
                    rootNodes: [rootNode]
                },
                {
                    edge: {
                        of: "depends",
                        traversal: {
                            reverseDirection: false
                        }
                    }
                },
                {
                    sortBy: {
                        key: {
                            connectivity: {
                                of: "depends",
                                type: weightDecayMode
                            }
                        },
                        desc: true
                    }
                },
                {
                    limit: {
                        depth
                    }
                },
                {
                    limit: {
                        batchSize: limit
                    }
                }
            ]
        }
    };

    const lhsResponse = await postRequest(
        url,
        config,
        (e) => { throw e; }
    );

    // Find dependents instead
    config.graph.constraints[1].edge.traversal.reverseDirection = true;

    const rhsResponse = await postRequest(
        url,
        config,
        (e) => { throw e; }
    );

    return formatTreeData(rootNode, lhsResponse.data, rhsResponse.data);
};

export const getMockTreeSimple = async () => {
    return MOCK_TREE;
};