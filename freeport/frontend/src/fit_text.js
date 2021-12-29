/// A function to fit text into a square and find the radius of the enclosing circle.
/// Adapted from https://observablehq.com/@mbostock/fit-text-to-circle

// Prepare an immutable context for measuring text width
const context = Object.freeze(document.createElement("canvas").getContext("2d"));
const measureWidth = (text) => {
    return context.measureText(text).width;
};

export function fitTextInSquare(text, fontSize) {
    // Split text into words by '-'
    const words = text.split(/\s+|(?<=-)/);
    if(!words[words.length - 1]) words.pop();
    if(!words[0]) words.shift();

    const lineHeight = fontSize + 0.2; // Add 0.2 for line spacing

    const targetWidth = Math.sqrt(measureWidth(text.trim()) * lineHeight);

    // Wrap text into lines
    let line = {};
    let lineWidth0 = Infinity;
    const lines = [];
    for (let i = 0, n = words.length; i < n; ++i) {
        let lineText1 = (line ? line.text : "") + words[i];
        let lineWidth1 = measureWidth(lineText1);
        if ((lineWidth0 + lineWidth1) / 2 < targetWidth) {
            line.width = lineWidth0 = lineWidth1;
            line.text = lineText1;
        } else {
            lineWidth0 = measureWidth(words[i]);
            lines.push(words[i]);
        }
    }

    return lines;
}