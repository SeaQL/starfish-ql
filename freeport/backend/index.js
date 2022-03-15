const { scrape } = require("./src/scrape/main");

scrape({
    shouldLog: true
})
.catch(console.error);