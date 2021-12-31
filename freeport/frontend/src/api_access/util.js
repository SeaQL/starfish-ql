export const makeConstructUrl = (repo_url) => (endpoint, queryParams) => {
    let url = repo_url + "/" + endpoint + "?";
    if (queryParams !== undefined) {
        url += queryParams.join("&");
    }
    return url;
};

export const getRequestJson = async (
    url,
    err = (e) => console.error("Error " + e)
) => {
    const response = await fetch(url);

    if (!response.ok) {
        err(response.status);
        return false;
    }

    return await response.json();
};

export const sleep = (ms) => {
    return new Promise((resolve) => setTimeout(resolve, ms));
};