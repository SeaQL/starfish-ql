import { constructUrl } from "./url";
import { postRequest } from "./util";

export async function test_api() {

    await create_entity();

}

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