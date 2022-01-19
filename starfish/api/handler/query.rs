use crate::api::ErrorResponder;
use crate::query::{GraphData, NodeWeight, TreeData};
use crate::{api::db::pool::Db, query::Query};
use rocket::serde::json::Json;
use rocket::{get, routes};
use sea_orm_rocket::Connection;

pub fn routes() -> Vec<rocket::Route> {
    routes![get_graph, get_tree]
}

#[get("/get-graph?<top_n>&<limit>&<depth>&<weight>")]
async fn get_graph(
    conn: Connection<'_, Db>,
    top_n: Option<i32>,
    limit: Option<i32>,
    depth: Option<i32>,
    weight: Option<String>,
) -> Result<Json<GraphData>, ErrorResponder> {
    let db = conn.into_inner();
    let top_n = top_n.unwrap_or(0);
    let limit = limit.unwrap_or(0);
    let depth = depth.unwrap_or(0);
    let weight = weight.unwrap_or_else(Default::default);
    let weight: NodeWeight = serde_json::from_str(&weight).unwrap_or(NodeWeight::Simple);

    println!("Weight: {:?}", weight);

    Ok(Json(
        Query::get_graph(db, top_n, limit, depth, weight)
            .await
            .map_err(Into::into)?,
    ))
}

#[get("/get-tree?<root_node>&<limit>&<depth>&<weight>")]
async fn get_tree(
    conn: Connection<'_, Db>,
    root_node: Option<String>,
    limit: Option<i32>,
    depth: Option<i32>,
    weight: Option<String>,
) -> Result<Json<TreeData>, ErrorResponder> {
    let db = conn.into_inner();
    let root_node = root_node.unwrap_or_else(|| "serde".to_owned());
    let limit = limit.unwrap_or(0);
    let depth = depth.unwrap_or(0);
    let weight = weight.unwrap_or_else(Default::default);
    let weight: NodeWeight = serde_json::from_str(&weight).unwrap_or(NodeWeight::Simple);

    Ok(Json(
        Query::get_tree(db, root_node, limit, depth, weight)
            .await
            .map_err(Into::into)?,
    ))
}
