const { promisedExec, promisedExecInFolder } = require("./util");

/*
    Format of Metadata:
    .meta [
        <40-digit last commit hash>
    ]
*/
const NUM_METADATA = 1;

/// Assumes that a folder exists at 'path' and `touch 'path'` does not throw.
/// If a file already exists at 'path', it is appended.
const createMetadata = async (
    path,
    shouldLog,
    repo_path
) => {
    // Create a file at 'path'
    await promisedExec(`touch ${path}`);

    // Write entries to 'path'

    // Store the hash of the most recent commit
    const mostRecentCommitHash = (await promisedExecInFolder(repo_path, "git rev-parse --verify HEAD"))[0];
    shouldLog && console.log(`Most recent commit hash found: ${mostRecentCommitHash}`);
    await promisedExec(`echo ${mostRecentCommitHash} >> ${path}`);

    shouldLog && console.log(`Metadata file created at ${path}`);
};

const verifyMetadata = async (path, shouldLog) => {
    // Test for existence
    try {
        await promisedExec(`ls ${path}`);
    } catch (e) {
        shouldLog && console.log(`Metadata does not exist at ${path}.`);
        return false;
    }

    // Test for correctness
    const metadata = await promisedExec(`cat ${path}`);
    if (metadata.length !== NUM_METADATA) {
        shouldLog && console.log("Bad number of metadata.");
        return false;
    }

    const lastCommitHash = metadata[0];
    if (lastCommitHash.length !== 40) {
        shouldLog && console.log("Bad last commit hash.");
        return false;
    }

    shouldLog && console.log("Metadata is verified");
    return true;
};

module.exports = {
    createMetadata,
    verifyMetadata,
};