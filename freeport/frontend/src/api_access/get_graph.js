import { formatGraphData, postRequest } from "./util";
import { constructUrl } from "./url";
import MOCK_GRAPH_SIMPLE from "../data/mock_graph/simple.json";

export const getGraph = async (topN, limit, depth, weightDecayMode) => {
    const url = constructUrl("query");

    const res = await postRequest(
        url,
        {
            graph: {
                of: "crate",
                constraints: [
                    {
                        rootNodes: [
                            "sea-orm"
                        ]
                    },
                    {
                        edge: {
                            of: "depends"
                        }
                    },
                    {
                        limit: {
                            "depth": 3
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