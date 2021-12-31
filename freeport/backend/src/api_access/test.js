const { constructUrl } = require("./url");
const { postRequest } = require("./util");

module.exports = {
    test_api: async () => {
        await create_entity();
    },
};

async function create_entity() {
    const data = {
        "name": "crate",
        "attributes": [
            {
                "name": "version",
                "datatype": "String"
            },
            {
                "name": "owner",
                "datatype": "String"
            },
            {
                "name": "last_update",
                "datatype": "String"
            }
        ]
    };

    await postRequest(
        constructUrl("schema/create-entity"),
        data
    )
}