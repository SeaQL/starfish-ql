const axios = require("axios").default;

export const postRequest = (url, data, config = {}) => {
    return await axios.post(url, data, config);
};

export const sleep = (ms) => {
    return new Promise((resolve) => setTimeout(resolve, ms));
};