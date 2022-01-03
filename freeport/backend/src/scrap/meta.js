const { promisedExec } = require("./util");

/*
    Format of Metadata:
    .meta [
        <40-digit last commit hash>
    ]
*/
const NUM_METADATA = 1;

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
    verifyMetadata
};