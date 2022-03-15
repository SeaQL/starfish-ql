const BASE_URL = "http://localhost:8000";

export const constructUrl = (endpoint) => {
    let url = '';
    if (process.env.API_BASE_URL) {
        url += process.env.API_BASE_URL;
    } else {
        url += BASE_URL;
    }
    url += "/" + endpoint + "?";

    console.log(process.env.API_BASE_URL);

    return url;
};