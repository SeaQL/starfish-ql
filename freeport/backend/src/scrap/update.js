const { insertDataIntoDatabase } = require("../api_access/main");
const { readFileLineByLine } = require("./file_reader");
const { createMetadata } = require("./meta");
const { promisedExecInFolder } = require("./util");

const updateScrap = async (shouldLog, metadata, dataPath, metaName, repoPath) => {

    const lastCommitHash = metadata.lastCommitHash;
    shouldLog && console.log(`Last commit hash found: ${lastCommitHash}`);

    const mostRecentCommitHash = (await promisedExecInFolder(repoPath, "git rev-parse --verify HEAD"))[0];
    shouldLog && console.log(`Most recent commit hash found: ${mostRecentCommitHash}`);

    if (lastCommitHash === mostRecentCommitHash) {
        shouldLog && console.log("Commit hashes match. No need to update.");
        return;
    }

    // Do a git diff from last commit to the most recent commit
    (await promisedExecInFolder(repoPath, `git diff ${lastCommitHash}...${mostRecentCommitHash} > ../${dataPath}diff`))
    
    const dataMap = new Map();

    await readFileLineByLine(dataPath + "diff", (line) => {
        // Keep only lines starting with exactly 1 '+'
        if (line.length >= 2 && line[0] === '+' && line[1] !== '+') {
            // Remove the '+'
            const datum = JSON.parse(line.substring(1));
            dataMap.set(datum.name, datum);
        }
    });
    const data = Array.from(dataMap.values());

    shouldLog && console.log("Updating crates: ", data.map((datum) => datum.name));
    await insertDataIntoDatabase(data, { shouldLog });

    // Update metadata when everything is ready
    shouldLog && console.log("Updating metadata...");
    await createMetadata(metadata.filePath, shouldLog, repoPath);

    // Clean up
    await promisedExecInFolder(dataPath, `ls | grep -v '${metaName}' | xargs rm`);
};

module.exports = {
    updateScrap
};