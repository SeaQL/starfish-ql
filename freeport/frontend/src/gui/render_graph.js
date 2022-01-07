import * as d3 from "d3";
import { addWrappedTextToNodeAndSetTextRadius } from "./add_text_to_node";
import { createNodes } from "./create_nodes";
import { addDragBehavior } from "./drag";
import { addZoomBehavior } from "./zoom";
import { createInfobox, updateInfobox } from "./infobox";

/*
'data' must follow this format:
    data = {
        nodes: [
            { id: "A", weight: 15 },
            { id: "B", weight: 25 },
        ],
        links: [
            { source: "A", target: "B" },
        ]
    };
The behavior is undefined unless all id's in 'data.nodes' are unique.

'containerElem' is the container (HTMLElement) that contains the constructed svg graph.
*/
export function renderGraph(
    data,
    containerElem,
    {
        textDelimiters = "-",
        minFontSize = 12,
    } = {}
) {
    // set the dimensions and margins of the graph
    const margin = { top: 20, right: 20, bottom: 20, left: 20 },
          width = 1000 - margin.left - margin.right,
          height = 700 - margin.top - margin.bottom;
    
    const svg = d3.select(containerElem)
        .append("svg")
        .attr("width", width + margin.left + margin.right)
        .attr("height", height + margin.top + margin.bottom);
    
    const group = svg.append("g")
        .attr("transform", "translate(" + margin.left + "," + margin.top + ")");
    
    // Initialize the links
    const link = group
        .selectAll("line")
        .data(data.links)
        .enter()
        .append("line")
        .style("stroke", "#aaa");
    
    // Initialize the nodes
    const node = createNodes(group, data.nodes);
    
    // Draw circles for the nodes
    node.append("circle")
        .attr("r", (d) => d.weight)
        .style("fill", "#69b3a2");
    
    // Add names to the nodes
    addWrappedTextToNodeAndSetTextRadius(
        node,
        (d) => d.id,
        (d) => d.weight,
        (_) => "Fira Code, monospace",
        textDelimiters,
        minFontSize,
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
    
    const simulation = d3.forceSimulation(data.nodes)
        .force("link", d3.forceLink()
            .id((d) => d.id)
            .links(data.links)
            .distance((d) => (d.source.weight + d.target.weight) * 1.2)
        )
        .force("collision", d3.forceCollide().radius((d) => d.weight + 5))
        .force("charge", d3.forceManyBody())
        .force("center", d3.forceCenter(width / 2, height / 2))
        .on("tick", () => {
            link.attr("x1", (d) => d.source.x)
                .attr("y1", (d) => d.source.y)
                .attr("x2", (d) => d.target.x)
                .attr("y2", (d) => d.target.y);

            // Move circles
            node.select("circle")
                .attr("cx", (d) => d.x)
                .attr("cy", (d) => d.y);

            // Move names
            node.select("text")
                .attr("transform", (d) => `translate(${d.x}, ${d.y}) scale(${d.weight / d.textRadius})`);
        });

    addDragBehavior(node, simulation);
    addZoomBehavior(group, svg, width, height);
};