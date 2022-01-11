import * as d3 from "d3";
import { addDragBehavior } from "./drag";
import { addWrappedTextToNodeAndSetTextRadius } from "./add_text_to_node";
import { addZoomBehavior } from "./zoom";
import { createNodes } from "./create_nodes";
import { createInfobox, updateInfobox } from "./infobox";
import { highlightConnectedNodesAndLinks, resetAllHighlight } from "./highlight";

const ColorScheme = [
    "#69b3a2", // Root
    "#7ac931", // Dependency
    "#288cbd", // Dependent
];

// Denotes which side a node belongs to, relative to the **root** node.
export const TreeElemType = {
    Root: 0,  // Centered
    Dependency: 1,  // To the Left
    Dependent: 2,  // To the Right
    NUM_TREE_ELEM_TYPE: 3, // Make sure it is the last variant with the largest, consecutive value
};

if (ColorScheme.length !== TreeElemType.NUM_TREE_ELEM_TYPE) {
    console.error("Number of colors in ColorScheme does not match with number of tree element types.");
}

export function renderTree(
    data,
    containerElem,
    {
        nodeCircleRadius = 12,
        textDelimiters = "-"
    } = {}
) {
    // set the dimensions and margins of the graph
    const margin = { top: 20, right: 20, bottom: 20, left: 20 },
        width = 1000 - margin.left - margin.right,
        height = 700 - margin.top - margin.bottom;

    const center = { x: width / 2, y: height / 2 };

    const svg = d3.select(containerElem)
        .append("svg")
        .attr("width", width + margin.left + margin.right)
        .attr("height", height + margin.top + margin.bottom)

    const group = svg.append("g")
        .attr("transform", "translate(" + margin.left + "," + margin.top + ")");

    // Initialize the links
    const link = group
        .selectAll("line")
        .data(data.links)
        .enter()
        .append("line")
        .style("stroke", (d) => ColorScheme[d.type]);

    // Initialize the nodes
    const node = createNodes(group, data.nodes)
        .attr("", (d) => {
            if (d.type === TreeElemType.Root) {
                d.fx = center.x;
                d.fy = center.y;
            } else {
                d.x = center.x;
                d.y = center.y;
            }
            return null;
        });

    // Draw circles for the nodes
    node.append("circle")
        .attr("r", 20)
        .style("fill", (d) => ColorScheme[d.type]);

    // Add names to the nodes
    addWrappedTextToNodeAndSetTextRadius(
        node,
        (d) => d.id,
        (_) => nodeCircleRadius,
        (_) => "Fira Code, monospace",
        textDelimiters
    );

    // Setup infobox
    const infobox = createInfobox(svg);
    node.on("click.info", function (event, d) {
        if (event.defaultPrevented) return;
        
        updateInfobox(
            infobox,
            [
                "Id: " + d.id,
                "Testing1",
                "Testing2 Hihi"
            ],
        );
    });

    // Setup highlight behavior
    node.on(
        "mouseover.highlight",
        (_, d) => highlightConnectedNodesAndLinks(d.id, node, link)
    );
    node.on("mouseout.resetHighlight", (_) => resetAllHighlight(node, link));

    const simulation = d3.forceSimulation(data.nodes)
        .force("side", (alpha) => { // Dependencies to the left; Dependents to the right
            data.nodes.forEach(node => {
                if (node.type === TreeElemType.Dependency) {
                    node.vx = -Math.abs(node.vx);
                } else if (node.type === TreeElemType.Dependent) {
                    node.vx = Math.abs(node.vx);
                }
            });
        });

    let isExtraForcesAdded = false;

    const getSourceX = (d) => d.source.x;
    const getSourceY = (d) => d.source.y;
    const getTargetX = (d) => d.target.x;
    const getTargetY = (d) => d.target.y;
    const getX = (d) => d.x;
    const getY = (d) => d.y;
    const translateAndScale = (d) => `translate(${d.x}, ${d.y}) scale(${nodeCircleRadius / d.textRadius})`;

    simulation.on("tick", function() {
        link.attr("x1", getSourceX)
            .attr("y1", getSourceY)
            .attr("x2", getTargetX)
            .attr("y2", getTargetY);

        // Move circles
        node.select("circle")
            .attr("cx", getX)
            .attr("cy", getY);

        // Move names
        node.select("text")
            .attr("transform", translateAndScale);

        if (this.alpha() < 0.5 && !isExtraForcesAdded) {
            this.force("charge", d3.forceManyBody()
                    .strength(-200) // Push away each other
                )
                .force("link", d3.forceLink()
                    .id((d) => d.id)
                    .links(data.links)
                    .distance(50)
                    .strength(1)
                )
                // .force("side", null)

            isExtraForcesAdded = true;
        }
    });

    addDragBehavior(node, simulation);
    addZoomBehavior(group, svg, width, height);
};