const { promisedExec, promisedExecInFolder } = require("./util");
const { verifyMetadata } = require("./meta");
const { initialScrap } = require("./initial");
const { updateScrap } = require("./update");

const scrap = async ({
    shouldLog = true,
}) => {
    const REPO_NAME = "crates.io-index";
    const REPO_URL = "git clone https://github.com/rust-lang/crates.io-index.git";
    const DATA_PATH = "data/"; // Scrapped data storage, must end with '/'
    const META_NAME = "meta";

    const folders = await promisedExec("ls");

    // DEBUG ONLY - Remove data
    if (folders.find((folder) => DATA_PATH.search(folder))) {
        await promisedExec(`rm -rf ${DATA_PATH}`);
    }

    // Make sure the repo is ready.
    if (!folders.find((folder) => folder === REPO_NAME)) {
        // Clone the repo
        shouldLog && console.log("Repo not found, cloning...");
        await promisedExec(REPO_URL);
    } else {
        shouldLog && console.log("Repo found");
    }

    // Make sure the repo is up to date
    await promisedExecInFolder(REPO_NAME, "git checkout master && git pull && git merge --ff-only");

    // Branch on whether the metadata file is verified
    if (await verifyMetadata(DATA_PATH + META_NAME, shouldLog)) {
        await updateScrap(shouldLog);
    } else {
        await initialScrap(shouldLog, DATA_PATH, META_NAME, REPO_NAME);
    }
};

module.exports = {
    scrap
};