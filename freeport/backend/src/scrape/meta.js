const { promisedExec, promisedExecInFolder } = require("./util");

/*
    Format of Metadata:
    .meta [
        <40-digit last commit hash>
    ]
*/
const NUM_METADATA = 1;

/// Assumes that a folder exists at 'path' and `touch 'path'` does not throw.
/// If a file already exists at 'path', it is removed before a new one is created.
const createMetadata = async (
    path,
    shouldLog,
    repoPath
) => {
    // Create a file at 'path'
    try {
        await promisedExec(`rm ${path}`);
    } catch (e) {}
    await promisedExec(`touch ${path}`);

    // Write entries to 'path'

    // Store the hash of the most recent commit
    const mostRecentCommitHash = (await promisedExecInFolder(repoPath, "git rev-parse --verify HEAD"))[0];
    shouldLog && console.log(`Most recent commit hash found: ${mostRecentCommitHash}`);
    await promisedExec(`echo ${mostRecentCommitHash} >> ${path}`);

    shouldLog && console.log(`Metadata file created at ${path}`);
};

const parseMetadata = async (path, shouldLog) => {
    // Test for existence
    try {
        await promisedExec(`ls ${path}`);
    } catch (e) {
        shouldLog && console.log(`Metadata does not exist at ${path}.`);
        return null;
    }

    // Test for correctness
    const metadata = { filePath: path };
    const metadataLines = await promisedExec(`cat ${path}`);

    if (metadataLines.length !== NUM_METADATA) {
        shouldLog && console.log("Bad number of metadata.");
        return null;
    }

    metadata.lastCommitHash = metadataLines[0];
    if (metadata.lastCommitHash.length !== 40) {
        shouldLog && console.log("Bad last commit hash.");
        return null;
    }

    shouldLog && console.log("Metadata is verified");
    return metadata;
};

module.exports = {
    createMetadata,
    parseMetadata,
};