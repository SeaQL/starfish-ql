use rocket::serde::json::Json;
use rocket::{get, post, routes};
use sea_orm_rocket::Connection;
use starfish_core::mutate::Mutate;

use crate::{db::pool::Db, ErrorResponder};
use starfish_core::lang::{MutateInsertContentJson, MutateJson, MutateSelectorJson, SchemaJson};
use starfish_core::schema::Schema;

pub fn routes() -> Vec<rocket::Route> {
    routes![schema, mutate, query,]
}

#[post("/schema", data = "<input_data>")]
async fn schema(
    conn: Connection<'_, Db>,
    input_data: Json<SchemaJson>,
) -> Result<(), ErrorResponder> {
    let db = conn.into_inner();
    let schema_json = input_data.clone();

    Schema::define_schema(db, schema_json)
        .await
        .map_err(Into::into)?;

    Ok(())
}

#[post("/mutate?<upsert>", data = "<input_data>")]
async fn mutate(
    conn: Connection<'_, Db>,
    input_data: Json<MutateJson>,
    upsert: bool,
) -> Result<(), ErrorResponder> {
    let db = conn.into_inner();
    let mutate_json = input_data.clone();

    Mutate::mutate(db, mutate_json, upsert)
        .await
        .map_err(Into::into)?;

    Ok(())
}

#[get("/query")]
async fn query(conn: Connection<'_, Db>) -> Result<Json<String>, ErrorResponder> {
    Ok(Json("Hello Query!".to_owned()))
}
