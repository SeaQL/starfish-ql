const { resetSchema } = require("./src/api_access/reset_schema");
const { scrap } = require("./src/scrap/main");

resetSchema()
.then(() => scrap({
    shouldLog: true
}))
.catch(console.error);