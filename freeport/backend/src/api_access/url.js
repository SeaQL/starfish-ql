const BASE_URL = process.env.API_BASE_URL ?? "http://127.0.0.1:8000";

module.exports = {
    constructUrl: (endpoint, queryParams = {}) => {
        let url = BASE_URL + "/" + endpoint + "?";

        let params = [];
        for (const [k, v] of Object.entries(queryParams)) {
            params.push(`${k}=${v}`);
        }
        if (process.env.API_AUTH_KEY) {
            params.push(`auth=${process.env.API_AUTH_KEY}`);
        }

        return url + params.join('&');
    }
};