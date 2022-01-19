const { constructUrl } = require("./url");
const { postRequest } = require("./util");

// Precondition: all nodes and edges are already inserted
const calculateAllConnectivity = async () => {
    const url = constructUrl("mutate/cal-conn");
    await postRequest(url);
};

module.exports = {
    calculateAllConnectivity
};