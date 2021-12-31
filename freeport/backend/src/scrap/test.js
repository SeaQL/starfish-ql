const { promisedExec, promisedExecInFolder } = require("./util");

module.exports = {
    test_scrap: async ({
            log = true,
        }) => {

        const REPO_NAME = "crates.io-index";
        const REPO_URL = "git clone https://github.com/rust-lang/crates.io-index.git";

        // Make sure the repo is ready.
        if (!(await promisedExec("ls")).find((line) => line === REPO_NAME)) {
            // Clone the repo
            if (log) {
                console.log("Repo not found, cloning...");
            }
            await promisedExec(REPO_URL);
        } else {
            log && console.log("Repo found");
        }

        console.log(await promisedExecInFolder(REPO_NAME, "ls"));
    },
};