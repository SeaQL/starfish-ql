export function createInfobox(
    containingElem,
    {
        opacity = 0.5,
        cornerRadius = 5,
        topPadding = "10px",
        vAlign = "center"
    } = {}
) {
    const infobox = containingElem
        .append("g")
        .style("display", "none")
        .attr("class", "tip")
        .style("transform-origin", vAlign + " top")
        .style("--halfWidth", "50px")
        .style("transform", "translate(calc(50% - var(--halfWidth)), " + topPadding + ")")
        .on("click.close", function () {
            this.style.display = "none";
        });

    infobox.append("rect")
        .attr("fill", "black")
        .attr("opacity", opacity)
        .attr("rx", cornerRadius);

    infobox.append("text")
        .attr("dominant-baseline", "hanging")
        .style("fill", "#eeeeee");

    return infobox;
}

/// `contents`: array of strings, representing the lines to be displayed in the infobox
export function updateInfobox(
    infobox,
    contents,
    {
        fontSize = 16,
        padding = 10,
        lineSpacing = 6,
    } = {}
) {
    infobox.style("display", "block");

    const numLines = contents.length;
    const text = infobox.select("text");

    text.selectAll("tspan").remove();

    text.selectAll("tspan")
        .data(contents)
        .enter()
        .append("tspan")
        .style("font-size", fontSize + "px")
        .attr("x", padding)
        .attr("y", (_, i) => {
            return padding + i * (fontSize + lineSpacing);
        })
        .text((d) => d);

    let bb = text.node().getBoundingClientRect();
    bb = {
        width: bb.width + 2 * padding,
        height: bb.height + 2 * padding
    };
    infobox.select("rect")
        .attr("width", bb.width)
        .attr("height", bb.height);

    infobox.style("--halfWidth", (bb.width / 2) + "px");
}