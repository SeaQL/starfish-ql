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
    delimiter = "-",
    minFontSize = 12,
) {
    const allWrappedLines = []; // An array of arrays of wrapped lines (one array for one node)

    const actualFontSizeFn = (d) => Math.max(fontSizeFn(d), minFontSize)

    const textElem = node.append("text")
        .style("font-size", (d) => actualFontSizeFn(d) + "px")
        .style("font-family", (d) => fontFamilyFn(d))
        .style("pointer-events", "none")
        .style("text-anchor", "middle")
        .attr("", (d) => {
            allWrappedLines.push(fitTextInSquare(textFn(d), actualFontSizeFn(d), delimiter));
            return null;
        });

    textElem.selectAll("tspan")
        .data((d, i) => allWrappedLines[i].map((line) => {
            return {
                line,
                numLines: allWrappedLines[i].length,
                fontSize: actualFontSizeFn(d)
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
        d.textRadius = (bb.width + bb.height) / 2;
        if (d.textRadius === 0) {
            d.textRadius = minFontSize;
        }
    });

    return textElem;
}