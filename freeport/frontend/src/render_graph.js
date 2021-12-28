import * as d3 from "d3";
import { addDragBehavior } from "./drag";
import { addZoomBehavior } from "./zoom";

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
export function renderGraph(data, containerElem) {
    // set the dimensions and margins of the graph
    const margin = { top: 20, right: 20, bottom: 20, left: 20 },
          width = 400 - margin.left - margin.right,
          height = 400 - margin.top - margin.bottom;
    
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
    const node = group
        .selectAll("circle")
        .data(data.nodes)
        .enter()
        .append("g");
    
    // Draw circles for the nodes
    node.append("circle")
        .attr("r", (d) => d.weight)
        .style("fill", "#69b3a2");
    
    // Add names to the nodes
    node.append("text")
        .text((d) => d.id)
        .style("font-size", (d) => `${d.weight}px`)
        .style("font-family", "Fira Code, monospace")
        .style("pointer-events", "none")
        .attr("", function(d) { d.bb = this.getBoundingClientRect(); return null; });
    
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
                .attr("x", (d) => d.x - d.bb.width / 2)
                .attr("y", (d) => d.y + d.bb.height / 4);
        });

    addDragBehavior(node, simulation);
    addZoomBehavior(group, svg, width, height);
};