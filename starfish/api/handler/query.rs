use crate::api::ErrorResponder;
use crate::query::GraphData;
use crate::{api::db::pool::Db, query::Query};
use rocket::serde::json::Json;
use rocket::{get, routes};
use sea_orm_rocket::Connection;

pub fn routes() -> Vec<rocket::Route> {
    routes![get_graph]
}

#[get("/get-graph?<top_n>&<depth>")]
async fn get_graph(
    conn: Connection<'_, Db>,
    top_n: Option<i32>,
    depth: Option<i32>,
) -> Result<Json<GraphData>, ErrorResponder> {
    let db = conn.into_inner();

    Ok(Json(
        Query::get_graph(db, top_n.unwrap_or(10), depth.unwrap_or(10))
            .await
            .map_err(Into::into)?,
    ))
}
