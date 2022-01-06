const BASE_URL = "http://127.0.0.1:8000";

module.exports = {
    constructUrl: (endpoint, queryParams = {}) => {
        let url = BASE_URL + "/" + endpoint + "?";

        for (const [k, v] of Object.entries(queryParams)) {
            url += `${k}=${v}`;
        }

        return url;
    }
};