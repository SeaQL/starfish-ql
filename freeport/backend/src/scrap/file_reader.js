const fs = require("fs");
const readline = require("readline");

const createLineReader = (path) => {
    return readline.createInterface({
        input: fs.createReadStream(path),
        console: false,
    });
};

const readFileLineByLine = (path, handleLine) => {
    const lineReader = createLineReader(path);

    return new Promise((resolve, reject) => {
        lineReader.on("line", handleLine);
        lineReader.on("close", resolve);
    });
};

module.exports = {
    readFileLineByLine
};