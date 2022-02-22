const BASE_URL = "http://localhost:8000";

export const constructUrl = (endpoint) => {
    let url = BASE_URL + "/" + endpoint;

    return url;
};