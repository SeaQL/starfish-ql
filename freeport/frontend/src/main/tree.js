import { getMockTreeSimple, getTree } from "../api_access/get_tree";
import { normalizeData } from "../data/normalize";
import { Input, SubmitButton } from "../gui/input";
import { renderTree } from "../gui/render_tree";
import { clearChildNodes } from "../gui/util";

export const treeMain = async (GlobalConfig) => {

    const outputElem = document.getElementById(GlobalConfig.outputElemId);

    const run = () => {
        clearChildNodes(GlobalConfig.outputElemId);

        outputElem.innerText = "Loading...";

        // getTree(
        //     Input.treeRootNode.parseString(),
        //     Input.limit.parseInt(),
        //     Input.depth.parseInt(),
        // )
        getMockTreeSimple()
        .then((dataTree) => {

            outputElem.innerText = "";

            renderTree(
                dataTree,
                document.getElementById(GlobalConfig.outputElemId),
            );
        })
        .catch(console.error);
    };

    SubmitButton.addEventListener("click", run);
};