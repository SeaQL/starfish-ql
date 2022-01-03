const { promisedExec, promisedExecInFolder } = require("./util");
const { verifyMetaFile } = require("./meta");
const { initialScrap } = require("./initial");
const { updateScrap } = require("./update");

const scrap = async ({
    shouldLog = true,
}) => {
    const REPO_NAME = "crates.io-index";
    const REPO_URL = "git clone https://github.com/rust-lang/crates.io-index.git";

    // Make sure the repo is ready.
    if (!(await promisedExec("ls")).find((folder) => folder === REPO_NAME)) {
        // Clone the repo
        if (shouldLog) {
            console.log("Repo not found, cloning...");
        }
        await promisedExec(REPO_URL);
    } else {
        shouldLog && console.log("Repo found");
    }

    // Make sure the repo is up to date
    await promisedExecInFolder(REPO_NAME, "git pull && git merge --ff-only");

    // Store the hash of the most recent commit
    const mostRecentCommitHash = (await promisedExecInFolder(REPO_NAME, "git rev-parse --verify HEAD"))[0];

    // Branch on whether a .meta file is verified
    if (await verifyMetaFile("data/.meta", shouldLog)) {
        await updateScrap(shouldLog);
    } else {
        await initialScrap(shouldLog);
    }
};

module.exports = {
    scrap
};