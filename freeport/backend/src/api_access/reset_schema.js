const { constructUrl } = require("./url");
const { postRequest } = require("./util");

const defineSchema = async (reset = true) => {
    await postRequest(
        constructUrl("schema"),
        {
            reset,
            define: {
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
        }
    );
};

module.exports = {
    defineSchema
};