const { AsyncBatch } = require("./batch");
const { insertNodesBatch, insertEdgesBatch, createCrateNode, createDependsEdge } = require("./insert");

/// 'data' is obtained from the 'scrap/main' module.
const insertDataIntoDatabase = async (
    data,
    {
        batchReleaseThreshold = 1000,
        shouldLog = true
    } = {}
) => {
    const numData = data.length;

    const nodesBatch = new AsyncBatch(batchReleaseThreshold, insertNodesBatch, shouldLog);
    const edgesBatch = new AsyncBatch(batchReleaseThreshold, insertEdgesBatch, shouldLog);

    for (let i = 0; i < numData; ++i) {
        const datum = data[i];

        // Create own node
        await nodesBatch.push(
            createCrateNode(datum.name, {
                version: datum.vers,
            })
        );

        const depNames = new Set();
        for (let dep of datum.deps) {
            if (dep.kind === "dev" || depNames.has(datum.name)) {
                continue;
            }
            depNames.add(datum.name);
            // Create depends edge
            await edgesBatch.push(
                createDependsEdge(datum.name, dep.name)
            );
        }

        shouldLog
            && ((i + 1) % 100 === 0 || (i + 1) === numData)
            && console.log(`Adding to batch for insertion... ${i + 1}/${numData}`);
    };

    await nodesBatch.release();
    await edgesBatch.release();
}

module.exports = {
    insertDataIntoDatabase,
};