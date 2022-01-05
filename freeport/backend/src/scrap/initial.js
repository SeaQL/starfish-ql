const { insertDataIntoDatabase } = require("../api_access/main");
const { readFileLineByLine, readLastLineOfFile } = require("./file_reader");
const { createMetadata } = require("./meta");
const { promisedExec, promisedExecInFolder } = require("./util");

const initialScrap = async (shouldLog, dataPath, metaName, repoPath) => {
    shouldLog && console.log("Commencing initial scrap...");

    // Clear data and metadata
    shouldLog && console.log("Removing existing data and metadata if any...");
    try {
        await promisedExec(`rm -rf ${dataPath} && rm ${dataPath + metaName}`);
    } catch (e) {}

    // Create data
    shouldLog && console.log("Creating data folder...");
    await promisedExec(`mkdir -p ${dataPath}`);

    // Create necessary data files
    shouldLog && console.log(`Creating intermediate data files...`);
    await promisedExec(`touch ${dataPath}paths`);

    // Store all crate files in an array
    await promisedExec(`find ${repoPath}/* -type f >> ${dataPath}paths`);
    const allFilePaths = [];
    await readFileLineByLine(
        `${dataPath}paths`,
        (line) => {
            // There should be no "." in the file path
            if (!line.substring(repoPath.length).includes(".")) {
                allFilePaths.push(line);
            }
        });
    shouldLog && console.log("All file paths loaded.");

    // Store the last entry of each crate file into entries
    const numPaths = allFilePaths.length;
    const entries = []
    for (let i = 0; i < numPaths; ++i) {
        shouldLog
        && ((i+1) % 1000 === 0 || (i+1) === numPaths)
        && console.log(`Data entries loading... ${i+1}/${numPaths}`);
        
        const entry = await readLastLineOfFile(allFilePaths[i]);
        entries.push(JSON.parse(entry));
    };
    shouldLog && console.log(`${entries.length} data entries loaded from ${numPaths} paths.`);

    await insertDataIntoDatabase(entries, { shouldLog });

    // Create metadata when everything is ready
    shouldLog && console.log("Creating metadata...");
    await createMetadata(dataPath + metaName, shouldLog, repoPath);

    // Clean up
    await promisedExecInFolder(dataPath, `ls | grep -v '${metaName}' | xargs rm`);
};

module.exports = {
    initialScrap
};