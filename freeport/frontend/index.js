import { renderGraph } from "./src/render_graph";
import { renderTree, TreeNodeType } from "./src/render_tree";

const dataGraph = {
    nodes: [
        { id: "A", weight: 15 },
        { id: "B", weight: 20 },
        { id: "C", weight: 25 },
        { id: "D", weight: 30 },
        { id: "E", weight: 22 },
        { id: "F", weight: 22 },
    ],
    links: [ // source "depends on" target
        { source: "A", target: "B" },
        { source: "C", target: "B" },
        { source: "B", target: "D" },
        { source: "E", target: "F" },
        { source: "F", target: "E" },
    ]
};

renderGraph(dataGraph, document.getElementById("outputGraph"));

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

renderTree(dataTree, document.getElementById("outputTree"));