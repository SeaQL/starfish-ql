const fs = require("fs");
const readline = require("readline");

const createLineReader = (path) => {
    return readline.createInterface({
        input: fs.createReadStream(path),
        console: false,
    });
};

/// WARNING: Does not reject on errors.
const readFileLineByLine = (path, handleLine) => {
    const lineReader = createLineReader(path);

    return new Promise((resolve) => {
        lineReader.on("line", handleLine);
        lineReader.on("close", resolve);
    });
};

/// WARNING: Does not reject on errors.
const readLastLineOfFile = (path) => {
    const lineReader = createLineReader(path);
    let lastLine = "";

    return new Promise((resolve) => {
        lineReader.on("line", (line) => {
            lastLine = line;
        });
        lineReader.on("close", () => {
            resolve(lastLine)
        });
    });
}

const writeToEndOfFile = (path, content) => {
    return new Promise((resolve, reject) => {
        fs.appendFile(path, content, err => {
            if (err) {
                reject(err);
            } else {
                resolve();
            }
        });
    });
};

module.exports = {
    readFileLineByLine,
    readLastLineOfFile,
    writeToEndOfFile,
};