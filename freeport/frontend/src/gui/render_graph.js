import * as d3 from "d3";
import { addWrappedTextToNodeAndSetTextRadius } from "./add_text_to_node";
import { createNodes } from "./create_nodes";
import { addDragBehavior } from "./drag";
import { addZoomBehavior } from "./zoom";
import { createInfobox, updateInfobox } from "./infobox";
import { isColorLight, stringToColour } from "./util";
import { highlightConnectedNodesAndLinks, resetAllHighlight } from "./highlight";

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
        .attr("width", "100vw")
        .attr("height", "100vh")
        // .attr("width", width + margin.left + margin.right)
        // .attr("height", height + margin.top + margin.bottom)
        ;
    
    const group = svg.append("g")
        .attr("transform", "translate(" + margin.left + "," + margin.top + ")");

    svg.append("defs")
        .append("marker")
        .attr("id", "arrowhead")
        .attr("viewBox", "0 -5 10 10")
        .attr("orient", "auto")
        .attr("refX", 8)
        .attr("refY", 0)
        .attr("markerWidth", 9)
        .attr("markerHeight", 5)
        .append("svg:path")
        .attr("d", "M0,-5L10,0L0,5")
        .attr("fill", "#aaa")
        .attr("stroke", "none")
        .attr("stroke-width", "1.5px");
    
    // Initialize the links
    const link = group
        .selectAll("line")
        .data(data.links)
        .enter()
        .append("line")
        .style("stroke", "#aaa")
        .attr("marker-end", "url(#arrowhead)");
    
    // Initialize the nodes
    const node = createNodes(group, data.nodes);
    
    // Draw circles for the nodes
    node.append("circle")
        .attr("r", (d) => d.weight)
        .style("fill", (d) => {
            const backgroundColor = stringToColour(d.id);
            d.isBackgroundLight = isColorLight(backgroundColor);
            return backgroundColor;
        });
    
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
            {
                crate: d.id,
                weight: `Weight: ${Math.round(d.weight * 10) / 10}`,
            },
        );
    });

    // Setup highlight behavior
    node.on(
        "mouseover.highlight",
        (_, d) => highlightConnectedNodesAndLinks(d.id, node, link)
    );
    node.on("mouseout.resetHighlight", (_) => resetAllHighlight(node, link));

    const getSourceX = (d) => d.source.x;
    const getSourceY = (d) => d.source.y;
    const getTargetX = (d) => d.target.x;
    const getTargetY = (d) => d.target.y;
    const getX = (d) => d.x;
    const getY = (d) => d.y;
    const translateAndScale = (d) => `translate(${d.x}, ${d.y}) scale(${d.weight / d.textRadius})`;
    const getTargetNodeCircumferencePoint = (d) => {
        var radius = d.target.weight + 1;
        var dx = d.target.x - d.source.x;
        var dy = d.target.y - d.source.y;
        var gamma = Math.atan2(dy,dx);
        var tx = d.target.x - (Math.cos(gamma) * radius);
        var ty = d.target.y - (Math.sin(gamma) * radius);
        return [tx, ty]; 
    };
    
    const simulation = d3.forceSimulation(data.nodes)
        .force("link", d3.forceLink()
            .id((d) => d.id)
            .links(data.links)
            .strength(0.1)
        )
        .force("collision", d3.forceCollide().radius((d) => d.weight * 1.1))
        .force("charge", d3.forceManyBody().strength(-200))
        .force("center", d3.forceCenter(width / 2, height / 2))
        .on("tick", () => {
            link.attr("x1", getSourceX)
                .attr("y1", getSourceY)
                .attr("x2", (d) => getTargetNodeCircumferencePoint(d)[0])
                .attr("y2", (d) => getTargetNodeCircumferencePoint(d)[1]);

            // Move circles
            node.select("circle")
                .attr("cx", getX)
                .attr("cy", getY);

            // Move names
            node.select("text")
                .attr("transform", translateAndScale);
        });

    addDragBehavior(node, simulation, ["link", "charge"]);
    addZoomBehavior(group, svg, width, height);
};