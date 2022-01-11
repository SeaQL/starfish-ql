import * as d3 from "d3";

let tempForces = {};
const modifyForces = (simulation, forcesToDisable, forcesToRemove) => {
    tempForces = {};
    forcesToDisable.forEach((forceName) => {
        tempForces[forceName] = simulation.force(forceName);
        simulation.force(forceName, null);
    });
    forcesToRemove.forEach((forceName) => simulation.force(forceName, null));
};
const restoreForces = (simulation) => {
    for (let [name, force] of Object.entries(tempForces)) {
        if (name === "link") {
            continue;
        }
        simulation.force(name, force);
    }
};

export function addDragBehavior(element, simulation, forcesToDisable = [], forcesToRemove = []) {
    const start = function(event) {
        modifyForces(simulation, forcesToDisable, forcesToRemove);
        if (!event.active) {
            simulation.alphaTarget(0.1).restart();
        }
        event.subject.fx = event.subject.x;
        event.subject.fy = event.subject.y;
    };

    const drag = function (event) {
        event.subject.fx = event.x;
        event.subject.fy = event.y;
    };

    const end = function (event) {
        restoreForces(simulation);
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
