const axios = require("axios").default;

module.exports = {
    getRequest: async (url, config = {}) => {
        return await axios.get(url, config);
    },

    postRequest: async (url, data, config = {}) => {
        return await axios.post(url, data, config);
    },
    
    sleep: (ms) => {
        return new Promise((resolve) => setTimeout(resolve, ms));
    }
};