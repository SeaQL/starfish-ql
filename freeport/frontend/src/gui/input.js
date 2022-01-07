const inputElemIds = [
    "graphTopN",
    "treeRootNode",
    "limit",
    "depth"
];

export const Input = inputElemIds.reduce((obj, id) => {

    obj[id] = {};

    const elem = obj[id].elem = document.getElementById(id);

    if (elem !== null) {
        obj[id].parseInt = () => parseInt(elem.value);
        obj[id].parseString = () => elem.value;
    }

    return obj;

}, {});

Input.elems = Object.values(Input).map((input) => input.elem);