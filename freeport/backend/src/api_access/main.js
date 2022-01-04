const { insertCrateNode, insertDependsEdge } = require("./insert");

/// 'data' is obtained from the 'scrap/main' module.
const insertDataIntoDatabase = async (data, shouldLog = true) => {
    const numData = data.length;

    for (let i = 0; i < numData; ++i) {
        const datum = data[i];

        // Create own node
        await insertCrateNode(datum.name, {
            version: datum.vers,
        });

        const depNames = new Set();
        for (let dep of datum.deps) {
            if (dep.kind === "dev" || depNames.has(datum.name)) {
                continue;
            }
            depNames.add(datum.name);
            // Create depends edge
            await insertDependsEdge(datum.name, dep.name);
        }

        shouldLog
            && ((i + 1) % 100 === 0 || (i + 1) === numData)
            && console.log(`Inserting into Database... ${i + 1}/${numData}`);
    };
}

module.exports = {
    insertDataIntoDatabase,
};