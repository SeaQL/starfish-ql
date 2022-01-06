const BASE_URL = "http://localhost:8000";

export const constructUrl = (endpoint, queryParams = {}) => {
    let url = BASE_URL + "/" + endpoint + "?";

    for (const [k, v] of Object.entries(queryParams)) {
        url += `${k}=${v}&`;
    }

    return url;
};