import { getRequestJson } from "./util";
import { constructUrl } from "./url";

export const getGraphSimple = async (topN) => {
    const url = constructUrl(
        "query/get-graph",
        {
            top_n: topN,
            depth: 2,
        }
    );

    const graph = await getRequestJson(
        url,
        (e) => { throw e; }
    );

    return graph;
};