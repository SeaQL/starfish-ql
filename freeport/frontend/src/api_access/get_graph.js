import { getRequestJson } from "./util";
import { constructUrl } from "./url";
import MOCK_GRAPH_SIMPLE from "../data/mock_graph/simple.json";

export const getGraph = async (topN, limit, depth, weightDecayMode) => {
    const url = constructUrl(
        "query/get-graph",
        {
            top_n: topN,
            limit,
            depth,
            weight: weightDecayMode
        }
    );

    const graph = await getRequestJson(
        url,
        (e) => { throw e; }
    );

    return graph;
};

export const getMockGraphSimple = async () => {
    return MOCK_GRAPH_SIMPLE;
};