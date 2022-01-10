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
        switch (obj[id].elem.type) {
            case "number":
                obj[id].parseInt = () => parseInt(elem.value);
                break;
            case "text":
                obj[id].parseString = () => elem.value;
                break;
            default:
        }
    }

    return obj;

}, {});

export const SubmitButton = document.getElementById("submit");

Input.elems = Object.values(Input).map((input) => input.elem);