const { promisedExecInFolder } = require("../scrap/util");
const { AsyncBatch } = require("./batch");
const { insertNodesBatch, insertEdgesBatch, createCrateNode, createDependsEdge } = require("./insert");

const now = () => (new Date()).getTime();

/// 'data' is obtained from the 'scrap/main' module.
const insertDataIntoDatabase = async (
    data,
    batchReleaseThreshold,
    {
        shouldLog = true
    } = {}
) => {
    const startTime = now();

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

    const errors = [];
    const errorHandler = (e) => {
        errors.push(e);
        console.error(e);
    };
    const nodesBatch = new AsyncBatch(batchReleaseThreshold, insertNodesBatch, shouldLog, errorHandler);
    const edgesBatch = new AsyncBatch(batchReleaseThreshold, insertEdgesBatch, shouldLog, errorHandler);

    await nodesBatch.consumeArray(nodes, "nodes");
    await edgesBatch.consumeArray(edges, "edges");

    shouldLog && console.log(
        `Inserted ${nodes.length + edges.length} items into database in ${Math.round((now() - startTime) / 1000)}s with ${errors.length} errors caught.`
    );
    return errors;
}

const insertDataIntoDatabaseAndLogErrors = async (
    data,
    logPath,
    {
        batchReleaseThreshold = 3000,
        shouldLog = true
    } = {}
) => {
    const errors = await insertDataIntoDatabase(data, batchReleaseThreshold, { shouldLog });
    if (errors.length > 0) {
        await promisedExecInFolder(logPath, "touch errors")
        for (let e of errors) {
            await promisedExecInFolder(logPath, `echo "${JSON.stringify(e)}" >> errors`);
        }
    }
};

module.exports = {
    insertDataIntoDatabase,
    insertDataIntoDatabaseAndLogErrors,
};
