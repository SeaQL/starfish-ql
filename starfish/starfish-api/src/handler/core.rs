use rocket::{get, post, routes};
use rocket::serde::json::Json;
use sea_orm_rocket::Connection;

use crate::api::{ErrorResponder, db::pool::Db};
use crate::core::lang::SchemaJson;
use crate::schema::Schema;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        schema,
        mutate,
        query,
    ]
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

#[post("/mutate")]
async fn mutate(
    conn: Connection<'_, Db>,
) -> Result<Json<String>, ErrorResponder> {
    
    Ok(Json("Hello Mutate!".to_owned()))
}

#[get("/query")]
async fn query(
    conn: Connection<'_, Db>,
) -> Result<Json<String>, ErrorResponder> {
    
    Ok(Json("Hello Query!".to_owned()))
}
