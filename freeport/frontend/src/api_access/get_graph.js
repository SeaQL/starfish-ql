import { getRequestJson } from "./util";
import { constructUrl } from "./url";
import MOCK_GRAPH_SIMPLE from "../data/mock_graph/simple.json";

export const getGraphSimple = async (topN, limit, depth, weightFactor) => {
    const url = constructUrl(
        "query/get-graph",
        {
            top_n: topN,
            limit,
            depth,
            weight: weightFactor
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