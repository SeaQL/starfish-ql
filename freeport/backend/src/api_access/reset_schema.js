const { constructUrl } = require("./url");
const { postRequest } = require("./util");

const resetDatabase = async () => {
    await postRequest(
        constructUrl("schema"),
        {
            reset: true
        }
    );
};

const defineSchema = async () => {
    await postRequest(
        constructUrl("schema"),
        {
            entities: [
                {
                    name: "crate",
                    attributes: [
                        {
                            name: "version",
                            datatype: "String",
                        }
                    ]
                }
            ],
            relations: [
                {
                    name: "depends",
                    from_entity: "crate",
                    to_entity: "crate",
                    directed: true
                }
            ]
        }
    );
};

const resetSchema = async () => {
    await resetDatabase();
    await defineSchema();
};

module.exports = {
    resetSchema
};