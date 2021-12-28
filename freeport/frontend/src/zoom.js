import * as d3 from "d3";

export function addZoomBehavior(element, container, width, height) {
    container.call(d3.zoom()
        .extent([[0, 0], [width, height]])
        .scaleExtent([1, 8])
        .on("zoom", ({ transform }) => {
            element.attr("transform", transform)
        })
    );
}