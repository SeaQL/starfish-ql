const { constructUrl } = require("./url");
const { postRequest, getRequest } = require("./util");

const resetDatabase = async () => {
    await getRequest(
        constructUrl("util/reset")
    );
};

const createEntity = async () => {
    await postRequest(
        constructUrl("schema/create-entity"),
        {
            name: "crate",
            attributes: [
                {
                    "name": "version",
                    "datatype": "String"
                }
            ]
        }
    )
};

const createRelation = async () => {
    await postRequest(
        constructUrl("schema/create-relation"),
        {
            name: "depends",
            from_entity: "crate",
            to_entity: "crate",
            directed: true
        }
    )
};

const resetSchema = async () => {
    await resetDatabase();
    await createEntity();
    await createRelation();
};

module.exports = {
    resetSchema
};