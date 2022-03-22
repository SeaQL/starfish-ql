use rocket::serde::json::Json;
use rocket::{post, routes};
use sea_orm_rocket::Connection;
use starfish_core::lang::mutate::MutateJson;
use starfish_core::lang::query::{QueryJson, QueryResultJson};
use starfish_core::lang::schema::SchemaJson;
use starfish_core::mutate::Mutate;
use starfish_core::query::Query;

use crate::{check_auth_match, db::pool::Db, ErrorResponder};
use starfish_core::schema::Schema;

pub fn routes() -> Vec<rocket::Route> {
    routes![schema, mutate, query,]
}

#[post("/schema?<auth>", data = "<input_data>")]
async fn schema(
    conn: Connection<'_, Db>,
    input_data: Json<SchemaJson>,
    auth: Option<String>,
) -> Result<(), ErrorResponder> {
    check_auth_match(auth)?;

    let db = conn.into_inner();
    let schema_json = input_data.clone();

    Schema::define_schema(db, schema_json)
        .await
        .map_err(Into::into)?;

    Ok(())
}

#[post("/mutate?<auth>&<upsert>", data = "<input_data>")]
async fn mutate(
    conn: Connection<'_, Db>,
    input_data: Json<MutateJson>,
    auth: Option<String>,
    upsert: bool,
) -> Result<(), ErrorResponder> {
    check_auth_match(auth)?;

    let db = conn.into_inner();
    let mutate_json = input_data.clone();

    Mutate::mutate(db, mutate_json, upsert)
        .await
        .map_err(Into::into)?;

    Ok(())
}

#[post("/query", data = "<input_data>")]
async fn query(
    conn: Connection<'_, Db>,
    input_data: Json<QueryJson>,
) -> Result<Json<QueryResultJson>, ErrorResponder> {
    let db = conn.into_inner();
    let query_json = input_data.clone();

    Ok(Json(
        Query::query(db, query_json).await.map_err(Into::into)?,
    ))
}
