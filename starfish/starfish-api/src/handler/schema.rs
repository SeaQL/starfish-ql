use crate::db::pool::Db;
use crate::ErrorResponder;
use rocket::serde::json::Json;
use rocket::{post, routes};
use sea_orm_rocket::Connection;
use starfish_core::lang::{EntityJson, RelationJson};
use starfish_core::schema::Schema;

pub fn routes() -> Vec<rocket::Route> {
    routes![create_entity, create_relation]
}

#[post("/create-entity", data = "<input_data>")]
async fn create_entity(
    conn: Connection<'_, Db>,
    input_data: Json<EntityJson>,
) -> Result<(), ErrorResponder> {
    let db = conn.into_inner();
    let entity_json = input_data.clone();

    Schema::create_entity(db, entity_json)
        .await
        .map_err(Into::into)?;

    Ok(())
}

#[post("/create-relation", data = "<input_data>")]
async fn create_relation(
    conn: Connection<'_, Db>,
    input_data: Json<RelationJson>,
) -> Result<(), ErrorResponder> {
    let db = conn.into_inner();
    let relation_json = input_data.clone();

    Schema::create_relation(db, relation_json)
        .await
        .map_err(Into::into)?;

    Ok(())
}
