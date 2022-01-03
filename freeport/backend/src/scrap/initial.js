const { readFileLineByLine, writeToEndOfFile, readLastLineOfFile } = require("./file_reader");
const { createMetadata } = require("./meta");
const { promisedExec } = require("./util");

const initialScrap = async (shouldLog, dataPath, metaName, repo_path) => {
    shouldLog && console.log("Commencing initial scrap...");

    // Clear data and metadata
    shouldLog && console.log("Removing existing data and metadata if any...");
    try {
        await promisedExec(`rm -rf ${dataPath} && rm ${dataPath + metaName}`);
    } catch (e) {}

    // Create data and metadata
    shouldLog && console.log("Creating data folder and metadata file...");
    await promisedExec(`mkdir -p ${dataPath}`);
    await createMetadata(dataPath + metaName, shouldLog, repo_path);

    // Create necessary data files
    shouldLog && console.log(`Creating data files...`);
    await promisedExec(`touch ${dataPath}entries`);
    await promisedExec(`touch ${dataPath}paths`);

    // Store all crate files in an array
    await promisedExec(`find ${repo_path}/* -type f >> ${dataPath}paths`);
    const allFilePaths = [];
    await readFileLineByLine(
        `${dataPath}paths`,
        (line) => {
            // There should be no "." in the file path
            if (!line.substring(repo_path.length).includes(".")) {
                allFilePaths.push(line);
            }
        });
    shouldLog && console.log("All file paths loaded.");

    // Store the last entry of each crate file into entries
    const numFiles = allFilePaths.length;
    for (let i = 0; i < numFiles; ++i) {
        // Read from allFilePaths[i]
        let lastLine = await readLastLineOfFile(allFilePaths[i]);

        // Write to 'entries'
        await writeToEndOfFile(`${dataPath}entries`, lastLine + "\n");

        shouldLog
            && ((i+1) % 1000 === 0 || (i+1) === numFiles)
            && console.log(`Data entries loading... ${i+1}/${numFiles}`);
    }
    shouldLog && console.log("All data entries loaded.");
};

module.exports = {
    initialScrap
};