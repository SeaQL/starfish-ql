import { getMockTreeSimple, getTree } from "../api_access/get_tree";
import { Input, SubmitButton } from "../gui/input";
import { renderTree } from "../gui/render_tree";
import { clearChildNodes } from "../gui/util";

export const treeMain = async (GlobalConfig, callback) => {

    const outputElem = document.getElementById(GlobalConfig.outputElemId);

    clearChildNodes(GlobalConfig.outputElemId);

    getTree(
        Input.treeRootNode.parseValue(),
        Input.limit.parseValue(),
        Input.depth.parseValue(),
        Input.weightDecayMode.parseValue(),
    )
    // getMockTreeSimple()
    .then((dataTree) => {
        renderTree(
            dataTree,
            document.getElementById(GlobalConfig.outputElemId),
        );

        callback();
    })
    .catch(console.error);
};