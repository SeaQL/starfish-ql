import { formatGraphData, postRequest } from "./util";
import { constructUrl } from "./url";
import MOCK_GRAPH_SIMPLE from "../data/mock_graph.json";

export const getGraph = async (topN, limit, depth, weightDecayMode) => {
    const url = constructUrl("query");

    const { data: topNNodes} = await postRequest(
        url,
        {
            vector: {
                of: "crate",
                constraints: [
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
                        limit: topN
                    }
                ]
            }
        },
        (e) => { throw e; }
    );

    const res = await postRequest(
        url,
        {
            graph: {
                of: "crate",
                constraints: [
                    {
                        rootNodes: topNNodes.map((node) => node.name)
                    },
                    {
                        edge: {
                            of: "depends"
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
        },
        (e) => { throw e; }
    );

    return formatGraphData(res.data);
};

export const getMockGraphSimple = async () => {
    return MOCK_GRAPH_SIMPLE;
};