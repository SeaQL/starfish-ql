// const { test_api } = require("./src/api_access/test");
// test_api().catch(console.error);

const { test_scrap } = require("./src/scrap/test");
test_scrap({
    log: true
}).catch(console.error);