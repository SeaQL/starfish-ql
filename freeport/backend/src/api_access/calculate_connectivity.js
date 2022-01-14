const { constructUrl } = require("./url");
const { postRequest } = require("./util");

// Precondition: all nodes and edges are already inserted
const calculateAllConnectivity = async () => {
    // Simple connectivity should be automatically evaluated
    await calculateCompoundConnectivity();
};

const calculateCompoundConnectivity = async () => {
    const url = constructUrl("mutate/cal-compound-conn");
    await postRequest(url);
};

module.exports = {
    calculateAllConnectivity
};