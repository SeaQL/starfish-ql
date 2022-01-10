import { getRequestJson } from "./util";
import { constructUrl } from "./url";

export const getTree = async (rootNode, limit, depth) => {
    const url = constructUrl(
        "query/get-tree",
        {
            root_node: rootNode,
            limit,
            depth,
        }
    );

    const tree = await getRequestJson(
        url,
        (e) => { throw e; }
    );

    return tree;
};