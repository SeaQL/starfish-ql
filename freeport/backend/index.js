// const { test_api } = require("./src/api_access/test");
// test_api().catch(console.error);

const { scrap: test_scrap } = require("./src/scrap/main");
test_scrap({
    shouldLog: true
}).catch(console.error);