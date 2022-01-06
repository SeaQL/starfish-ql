const { scrap } = require("./src/scrap/main");

scrap({
    shouldLog: true
})
.catch(console.error);