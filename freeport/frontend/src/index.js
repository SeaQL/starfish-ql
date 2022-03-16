import { graphMain } from "./main/graph";
import { treeMain } from "./main/tree";

const GlobalConfig = {
    minWeight: 12,
    maxWeight: 128,
    outputElemId: "output",
};

const main = async () => {

    const documentId = document.body.id;

    switch (documentId) {
        case "index":
            return;
        case "graph":
            await graphMain(GlobalConfig);
            break;
        case "tree":
            await treeMain(GlobalConfig);
            break;
        default:
    }

}; // End of main()

main().catch(console.error);