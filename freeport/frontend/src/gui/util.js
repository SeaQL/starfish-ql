export const clearChildNodes = (elemId) => {
    const elem = document.getElementById(elemId);
    while (elem !== null && elem.hasChildNodes()) {
        elem.removeChild(elem.lastChild);
    }
};