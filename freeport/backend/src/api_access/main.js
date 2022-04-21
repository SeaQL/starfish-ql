const { writeToEndOfFile } = require("../scrape/file_io");
const { promisedExecInFolder } = require("../scrape/util");
const { AsyncBatch } = require("./batch");
const filterEdges = require("./filter_edges").default;
const { insertCrateNodesBatch, insertDependsEdgesBatch, createNode, createEdge } = require("./insert");

const now = () => (new Date()).getTime();

/// 'data' is obtained from the 'scrape/main' module.
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
    let edges = [];

    for (let i = 0; i < numData; ++i) {
        const datum = data[i];

        // Create own node
        nodes.push(
            createNode(datum.name, {
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
                createEdge(datum.name, depName)
            );
        }
    };
    const filterResult = filterEdges(edges, nodes);
    edges = filterResult.valid;
    const invalidEdges = filterResult.invalid;
    shouldLog && console.log(`${invalidEdges.length} invalid edges filtered out:\n`, invalidEdges);
    shouldLog && console.log(`Collected ${nodes.length} nodes and ${edges.length} edges.`);

    if (nodes.length === 0) {
        return { nodes, edges, errors: [] };
    }

    const errors = [];
    const errorHandler = (e) => {
        errors.push(e);
        console.error(e.response.data);
    };
    const nodesBatch = new AsyncBatch(batchReleaseThreshold, insertCrateNodesBatch, shouldLog, errorHandler);
    await nodesBatch.consumeArray(nodes, "nodes");
    
    if (edges.length !== 0) {
        const edgesBatch = new AsyncBatch(batchReleaseThreshold, insertDependsEdgesBatch, shouldLog, errorHandler);
        await edgesBatch.consumeArray(edges, "edges");
    }

    shouldLog && console.log(
        `Inserted ${nodes.length + edges.length} items into database in ${Math.round((now() - startTime) / 1000)}s with ${errors.length} errors caught.`
    );
    return { nodes, edges, errors };
}

const insertDataIntoDatabaseAndLog = async (
    data,
    logPath,
    {
        batchReleaseThreshold = 3000,
        shouldLog = true
    } = {}
) => {
    const result = await insertDataIntoDatabase(data, batchReleaseThreshold, { shouldLog });
    if (result.errors.length > 0) {
        for (let e of result.errors) {
            e.errMsg = e.response.data;
            e.tempToJSON = e.toJSON;
            e.toJSON = () => {
                const json = e.tempToJSON();
                json.logTime = new Date();
                json.errMsg = e.response.data;
                return json;
            };
        }
    }

    await promisedExecInFolder(logPath, "touch log.js");
    await writeToEndOfFile(logPath + "log.js",
        "// Some edges in this file may contain non-existent nodes (e.g. 'ptable').\n"
    );
    await writeToEndOfFile(logPath + "log.js", JSON.stringify(result) + ";\n\n");
};

module.exports = {
    insertDataIntoDatabase,
    insertDataIntoDatabaseAndLog,
};
