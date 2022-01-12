export const clearChildNodes = (elemId) => {
    const elem = document.getElementById(elemId);
    while (elem !== null && elem.hasChildNodes()) {
        elem.removeChild(elem.lastChild);
    }
};

// Source: https://stackoverflow.com/a/16348977/17454868
export const stringToColour = (str) => {
    let hash = 0;
    for (let i = 0; i < str.length; ++i) {
        hash = str.charCodeAt(i) + ((hash << 5) - hash);
    }
    let colour = '#';
    for (var i = 0; i < 3; ++i) {
        const value = (hash >> (i * 8)) & 0xFF;
        colour += ('00' + value.toString(16)).slice(-2);
    }
    return colour;
};

// Source: https://awik.io/determine-color-bright-dark-using-javascript/
export const isColorLight = (color) => {

    // Variables for red, green, blue values
    let r, g, b, hsp;

    // Check the format of the color, HEX or RGB?
    if (color.match(/^rgb/)) {

        // If RGB --> store the red, green, blue values in separate variables
        color = color.match(/^rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*(\d+(?:\.\d+)?))?\)$/);

        r = color[1];
        g = color[2];
        b = color[3];
    }
    else {

        // If hex --> Convert it to RGB: http://gist.github.com/983661
        color = +("0x" + color.slice(1).replace(
            color.length < 5 && /./g, '$&$&'));

        r = color >> 16;
        g = color >> 8 & 255;
        b = color & 255;
    }

    // HSP (Highly Sensitive Poo) equation from http://alienryderflex.com/hsp.html
    hsp = Math.sqrt(
        0.299 * (r * r) +
        0.587 * (g * g) +
        0.114 * (b * b)
    );

    // Using the HSP value, determine whether the color is light or dark
    return hsp > 127.5;
};