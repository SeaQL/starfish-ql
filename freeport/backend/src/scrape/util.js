const util = require('util');
const child_process = require("child_process");

const exec = util.promisify(child_process.exec);

const promisedExec = async (command, shouldPostprocessOutput = true) => {
    let result = {
        stdout: [],
        stderr: "",
    };

    try {
        result = await exec(command);
    } catch (e) {
        throw e.stderr;
    }

    return shouldPostprocessOutput ? postprocessOutput(result.stdout) : result.stdout;
};

const promisedExecInFolder = async (folder, command, shouldPostprocessOutput = true) => {
    return await promisedExec(`cd ${folder} && ${command}`, shouldPostprocessOutput);
};

/// 1. Split 'str' by newline.
/// 2. Trim leading and trailing whitespaces.
/// 3. Filter out empty strings.
function postprocessOutput(str) {
    return str
        .split("\n")
        .reduce((filtered, line) => {
            const trimmed = line.trim();
            if (trimmed !== "") {
                filtered.push(trimmed);
            }
            return filtered;
        }, []);
}

module.exports = {
    promisedExec,
    promisedExecInFolder
};