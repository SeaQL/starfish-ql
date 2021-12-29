/// A function to add text (wrapped in square) to a node (selected in d3) element at an **arbitrary** position.
/// And then the approximate radius of the text square is stored in the data of 'node' as an attribute called 'textRadius'.
/// 'textFn': (d) => string
/// 'fontSizeFn': (d) => number
/// 'fontFamilyFn': (d) => string
/// 'delimiter': **additional** delimiters apart from whitespaces (as a string)

import { fitTextInSquare } from "./fit_text";

export function addWrappedTextToNodeAndSetTextRadius(
    node,
    textFn,
    fontSizeFn,
    fontFamilyFn,
    delimiter = "-"
) {
    const allWrappedLines = []; // An array of arrays of wrapped lines (one array for one node)

    const textElem = node.append("text")
        .style("font-size", (d) => fontSizeFn(d) + "px")
        .style("font-family", (d) => fontFamilyFn(d))
        .style("pointer-events", "none")
        .style("text-anchor", "middle")
        .attr("", (d) => {
            allWrappedLines.push(fitTextInSquare(textFn(d), fontSizeFn(d), delimiter));
            return null;
        });

    textElem.selectAll("tspan")
        .data((d, i) => allWrappedLines[i].map((line) => {
            return {
                line,
                numLines: allWrappedLines[i].length,
                fontSize: fontSizeFn(d)
            }
        }))
        .enter().append("tspan")
        .attr("x", 0)
        .attr("y", (d, i) => {
            return (i - d.numLines / 2 + 0.8) * d.fontSize;
        })
        .text((d) => d.line);

    textElem.attr("", function (d) {
        const bb = this.getBoundingClientRect();
        d.textRadius = Math.sqrt((bb.width / 2) ** 2 + (bb.height / 2) ** 2);
    });

    return textElem;
}