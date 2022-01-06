use crate::api::ErrorResponder;
use crate::query::GraphData;
use crate::{api::db::pool::Db, query::Query};
use rocket::serde::json::Json;
use rocket::{get, routes};
use sea_orm_rocket::Connection;

pub fn routes() -> Vec<rocket::Route> {
    routes![get_graph]
}

#[get("/get-graph?<root_min_in_conn>&<root_min_out_conn>&<depth>")]
async fn get_graph(
    conn: Connection<'_, Db>,
    root_min_in_conn: Option<i32>,
    root_min_out_conn: Option<i32>,
    depth: Option<i32>,
) -> Result<Json<GraphData>, ErrorResponder> {
    let db = conn.into_inner();

    Ok(Json(
        Query::get_graph(
            db,
            root_min_in_conn.unwrap_or(4_000),
            root_min_out_conn.unwrap_or(4_000),
            depth.unwrap_or(10),
        )
        .await
        .map_err(Into::into)?,
    ))
}
