export function createNodes(containingElem, nodesData) {
    return containingElem
        .selectAll("circle")
        .data(nodesData)
        .enter()
        .append("g");
}