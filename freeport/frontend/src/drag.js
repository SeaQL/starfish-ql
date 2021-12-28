import * as d3 from "d3";
import { TreeNodeType } from "./render_tree";

export function addDragBehavior(element, simulation) {
    const start = function(event) {
        if (!event.active) {
            simulation.alphaTarget(0.3).restart();
        }
        event.subject.fx = event.subject.x;
        event.subject.fy = event.subject.y;
    };

    const drag = function(event) {
        event.subject.fx = event.x;
        event.subject.fy = event.y;
    };

    const end = function(event) {
        if (!event.active) {
            simulation.alphaTarget(0);
        }
        event.subject.fx = null;
        event.subject.fy = null;
    };
    
    const dragBehavior = d3.drag()
        .on("start", start)
        .on("drag", drag)
        .on("end", end);

    element.call(dragBehavior);
}
