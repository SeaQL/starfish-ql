import { graphMain } from "./main/graph";
import { treeMain } from "./main/tree";

const GlobalConfig = {
    minWeight: 12,
    maxWeight: 128,
    outputElemId: "output",
};

window.main = async (callback) => {

    const documentId = document.body.id;

    switch (documentId) {
        case "index":
            return;
        case "graph":
            await graphMain(GlobalConfig, callback);
            break;
        case "tree":
            await treeMain(GlobalConfig, callback);
            break;
        default:
    }

}; // End of main()