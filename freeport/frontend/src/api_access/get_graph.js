import { getRequestJson } from "./util";
import { constructUrl } from "./url";

export const getGraphSimple = async (topN, limit, depth) => {
    const url = constructUrl(
        "query/get-graph",
        {
            top_n: topN,
            limit,
            depth,
        }
    );

    const graph = await getRequestJson(
        url,
        (e) => { throw e; }
    );

    return graph;
};