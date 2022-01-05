const { AsyncBatch } = require("./batch");
const { insertNodesBatch, insertEdgesBatch, createCrateNode, createDependsEdge } = require("./insert");

/// 'data' is obtained from the 'scrap/main' module.
const insertDataIntoDatabase = async (
    data,
    {
        batchReleaseThreshold = 3000,
        shouldLog = true
    } = {}
) => {
    const numData = data.length;

    const nodes = [];
    const edges = [];

    for (let i = 0; i < numData; ++i) {
        const datum = data[i];

        // Create own node
        nodes.push(
            createCrateNode(datum.name, {
                version: datum.vers,
            })
        );

        const depNames = new Set();
        for (let dep of datum.deps) {
            // In cargo.toml, 'package' stores the true crate name of a dependency when an alias is given to it.
            const depName = dep.package !== undefined ? dep.package : dep.name;
            if (dep.kind === "dev" || depNames.has(depName)) {
                continue;
            }
            depNames.add(depName);
            // Create depends edge
            edges.push(
                createDependsEdge(datum.name, depName)
            );
        }
    };
    shouldLog && console.log(`Collected ${nodes.length} nodes and ${edges.length} edges.`);

    const nodesBatch = new AsyncBatch(batchReleaseThreshold, insertNodesBatch, shouldLog);
    const edgesBatch = new AsyncBatch(batchReleaseThreshold, insertEdgesBatch, shouldLog);

    await nodesBatch.consumeArray(nodes, "nodes");
    await edgesBatch.consumeArray(edges, "edges");
}

module.exports = {
    insertDataIntoDatabase,
};