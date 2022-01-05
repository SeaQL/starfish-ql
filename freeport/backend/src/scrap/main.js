const { promisedExec, promisedExecInFolder } = require("./util");
const { parseMetadata } = require("./meta");
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

    // Make sure the repo is ready.
    if (!folders.find((folder) => folder === REPO_NAME)) {
        // Clone the repo
        shouldLog && console.log("Repo not found, cloning...");
        await promisedExec(REPO_URL);
    } else {
        shouldLog && console.log("Repo found");
    }

    // Make sure the repo is up to date
    await promisedExecInFolder(REPO_NAME, "git checkout master && git pull");

    // Branch on whether the metadata file is verified
    const metadata = await parseMetadata(DATA_PATH + META_NAME, shouldLog);
    if (metadata === null) {
        await initialScrap(shouldLog, DATA_PATH, META_NAME, REPO_NAME);
    } else {
        await updateScrap(shouldLog, metadata, DATA_PATH, REPO_NAME);
    }
};

module.exports = {
    scrap
};