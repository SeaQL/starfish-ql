import { getRequestJson, makeConstructUrl, sleep } from "./util";

const REPO_URL = "https://api.github.com/repos/rust-lang/crates.io-index";
const constructUrl = makeConstructUrl(REPO_URL);

export async function runGitHubAPITests() {
    await retrieveAllFiles();
    // await inspectCommitsDiff();
}

async function retrieveAllFiles() {
    const repo = await getRequestJson(REPO_URL);
    console.dir(repo);

    const recentCommits = await getRequestJson(constructUrl("commits"));
    console.dir(recentCommits);

    const mostRecentCommitSha = recentCommits[0].commit.tree.sha;
    const treeAtMostRecentCommit = await getRequestJson(
        constructUrl(
            "git/trees/" + mostRecentCommitSha,
            {
                recursive: true
            }
        )
    );
    console.dir(treeAtMostRecentCommit);
    const blobsOnly = treeAtMostRecentCommit.tree.filter((node) => !node.path.includes(".") && node.type === "blob");
    console.dir(blobsOnly);

    const testPath = "1/c";
    const testContent = await getRequestJson(
        `https://raw.githubusercontent.com/rust-lang/crates.io-index/master/${testPath}`
    );
    console.log(testContent);
}

/// Iteratively fetch all pages from an endpoint and collapse them into one large array.
async function fetchPagesIterative(endpoint, pageArrayFromDataFn = (data) => data) {
    const collapsedPages = [];

    for (let i = 1;;) {
        const url = constructUrl(
            endpoint,
            {
                page: i
            }
        );

        const data = await getRequestJson(url);

        const page = pageArrayFromDataFn(data);

        console.log("Page " + i, page);

        if (page === undefined || page.length === 0) {
            break;
        }

        collapsedPages.push(...page);
        ++i;
    }

    return collapsedPages;
}

async function inspectCommitsDiff() {
    const fromCommitSha = "dcb14b9012f15cf806bd0a61c982e9d6f76a7d63";

    const recentCommits = await getRequestJson(
        constructUrl(
            "commits"
        )
    );
    const toCommitSha = recentCommits[0].sha;

    if (fromCommitSha === toCommitSha) {
        return;
    }

    console.log(`Comparing diff from ${fromCommitSha} to ${toCommitSha}...`);
    const diffFiles = await fetchPagesIterative(
        `compare/${fromCommitSha}...${toCommitSha}`,
        (data) => data.files
    )
    // console.dir(diffFiles);
}