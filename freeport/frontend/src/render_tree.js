import * as d3 from "d3";

// Denotes which side a node belongs to, relative to the **root** node.
export const TreeNodeType = {
    Root: 0,  // Centered
    Dependency: 1,  // To the Left
    Dependent: 2,  // To the Right
};

export function renderTree(data, containerElem) {
    // set the dimensions and margins of the graph
    const margin = { top: 20, right: 20, bottom: 20, left: 20 },
        width = 400 - margin.left - margin.right,
        height = 400 - margin.top - margin.bottom;

    const center = { x: width / 2, y: height / 2 };

    const svg = d3.select(containerElem)
        .append("svg")
        .attr("width", width + margin.left + margin.right)
        .attr("height", height + margin.top + margin.bottom)
        .append("g")
        .attr("transform", "translate(" + margin.left + "," + margin.top + ")");

    // Initialize the links
    const link = svg
        .selectAll("line")
        .data(data.links)
        .enter()
        .append("line")
        .style("stroke", "#aaa");

    // Initialize the nodes
    const node = svg
        .selectAll("circle")
        .data(data.nodes)
        .enter()
        .append("g")
        .attr("", (d) => {
            if (d.type === TreeNodeType.Root) {
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
        .style("fill", "#69b3a2");

    // Add names to the nodes
    node.append("text")
        .text((d) => d.id)
        .style("font-size", (d) => d.weight + "px")
        .style("font-family", "Fira Code, monospace")
        .attr("", function (d) {
            const bbFull = this.getBoundingClientRect();
            d.bb = { width: bbFull.width, height: bbFull.height };
            return null;
        });

    const simulation = d3.forceSimulation(data.nodes)
        .force("side", (alpha) => { // Dependencies to the left; Dependents to the right
            data.nodes.forEach(node => {
                if (node.type === TreeNodeType.Dependency) {
                    node.vx = -Math.abs(node.vx);
                } else if (node.type === TreeNodeType.Dependent) {
                    node.vx = Math.abs(node.vx);
                }
            });
        });

    let addedExtraForces = false;

    simulation.on("tick", function() {
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

            if (this.alpha() < 0.5 && !addedExtraForces) {
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

                addedExtraForces = true;
            }
        });
}