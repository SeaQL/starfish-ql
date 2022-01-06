/// A function to fit text into a square, lines are only breaked at whitespaces or `delimiters`.
/// Adapted from https://observablehq.com/@mbostock/fit-text-to-circle

// Prepare an immutable context for measuring text width
const _context = Object.freeze(document.createElement("canvas").getContext("2d"));
const measureWidth = (text) => {
    return _context.measureText(text).width;
};

export function fitTextInSquare(text, fontSize, delimiters = "-") {
    // Split text into words by '-'
    const words = text.split(new RegExp(`\\s+|(?<=[${delimiters}])`));
    console.log(text);
    if(!words[words.length - 1]) words.pop();
    if(!words[0]) words.shift();

    const lineHeight = fontSize + 0.2; // Add 0.2 for line spacing

    const targetWidth = Math.sqrt(measureWidth(text.trim()) * lineHeight);

    // Wrap text into lines
    let line = words[0];
    const lines = [];
    for (let i = 1; i < words.length; ++i) {
        const lineWithWord = line + words[i];
        // Test if adding a word makes the line too long
        if (measureWidth(lineWithWord) >= targetWidth) {
            // Just push the current line
            lines.push(line);
            // Start again at next word
            line = words[i];
        } else {
            // Else, just absorb the word
            line = lineWithWord;
        }
    }
    if (line !== "") {
        lines.push(line);
    }

    return lines;
}