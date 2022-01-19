import { getRequestJson } from "./util";
import { constructUrl } from "./url";
import MOCK_TREE from "../data/mock_tree.json";

export const getTree = async (rootNode, limit, depth, weightDecayMode) => {
    const url = constructUrl(
        "query/get-tree",
        {
            root_node: rootNode,
            limit,
            depth,
            weight: weightDecayMode,
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