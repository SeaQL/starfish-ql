import { getRequestJson } from "./util";
import { constructUrl } from "./url";
import { MOCK_TREE } from "../data/mock";

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

export const getMockTreeSimple = async () => {
    return MOCK_TREE;
};