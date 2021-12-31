const axios = require("axios").default;

module.exports = {
    postRequest: async (url, data, config = {}) => {
        return await axios.post(url, data, config);
    },
    
    sleep: (ms) => {
        return new Promise((resolve) => setTimeout(resolve, ms));
    }
};