use rocket::{get, post, routes};
use rocket::serde::json::Json;
use sea_orm_rocket::Connection;

use crate::api::{ErrorResponder, db::pool::Db};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        schema,
        mutate,
        query,
    ]
}

#[post("/schema")]
async fn schema(
    conn: Connection<'_, Db>,
) -> Result<Json<String>, ErrorResponder> {
    
    Ok(Json("Hello Schema!".to_owned()))
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