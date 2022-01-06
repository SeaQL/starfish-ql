import { getGraphSimple } from "./src/api_access/get_graph";
import { normalizeData } from "./src/data/normalize";
import { renderGraph } from "./src/gui/render_graph";
import { renderTree, TreeNodeType } from "./src/gui/render_tree";

const WEIGHT_MIN = 12;
const WEIGHT_MAX = 128;

const main = async () => {

// const dataGraph = {
//     nodes: [
//         { id: "AAAA+AAAA+AAAAA", weight: 15 },
//         { id: "B", weight: 20 },
//         { id: "CCCCCCCCC", weight: 25 },
//         { id: "DDDDDDDDD-DDDDD", weight: 30 },
//         { id: "E", weight: 22 },
//         { id: "F", weight: 22 },
//     ],
//     links: [ // source "depends on" target
//         { source: "AAAA+AAAA+AAAAA", target: "B" },
//         { source: "CCCCCCCCC", target: "B" },
//         { source: "B", target: "DDDDDDDDD-DDDDD" },
//         { source: "E", target: "F" },
//         { source: "F", target: "E" },
//     ]
// };

const dataGraph = await getGraphSimple(10, 10, 5);
console.log("Raw: ", dataGraph);
normalizeData(
    dataGraph,
    (data) => data.nodes.map((node) => node.weight),
    (data, normalizedWeights) => {
        normalizedWeights.forEach((normalizedWeight, i) => {
            data.nodes[i].weight = normalizedWeight;
        });
    },
    {
        newMin: WEIGHT_MIN,
        newMax: WEIGHT_MAX,
    }
);
console.log("Normalized: ", dataGraph);

renderGraph(
    dataGraph,
    document.getElementById("outputGraph"),
    {
        textDelimiters: "-+",
        minFontSize: WEIGHT_MIN,
    }
);

const dataTree = {
    nodes: [
        { id: "A", type: TreeNodeType.Root },
        { id: "B", type: TreeNodeType.Dependency },
        { id: "E", type: TreeNodeType.Dependency },
        { id: "B1", type: TreeNodeType.Dependency },
        { id: "B11", type: TreeNodeType.Dependency },
        { id: "B2", type: TreeNodeType.Dependency },
        { id: "C", type: TreeNodeType.Dependent },
        { id: "C1", type: TreeNodeType.Dependent },
        { id: "D", type: TreeNodeType.Dependent },
    ],
    links: [ // source depends on target
        { source: "A", target: "B" },
        { source: "A", target: "E" },
        { source: "B", target: "B1" },
        { source: "B1", target: "B11" },
        { source: "B", target: "B2" },
        { source: "C", target: "A" },
        { source: "C1", target: "C" },
        { source: "D", target: "A" },
    ]
};

renderTree(
    dataTree,
    document.getElementById("outputTree"),
);

}; // End of main()

main().catch(console.error);