use crate::api::db::pool::Db;
use crate::api::ErrorResponder;
use crate::mutate::{ClearEdgeJson, EdgeJson, Mutate, NodeJson};
use rocket::futures::future::try_join_all;
use rocket::serde::json::Json;
use rocket::{post, routes};
use sea_orm_rocket::Connection;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        insert_node,
        insert_node_batch,
        insert_edge,
        insert_edge_batch,
        clear_edge
    ]
}

#[post("/insert-node", data = "<input_data>")]
async fn insert_node(
    conn: Connection<'_, Db>,
    input_data: Json<NodeJson>,
) -> Result<(), ErrorResponder> {
    let db = conn.into_inner();
    let node_json = input_data.clone();

    Mutate::insert_node(db, node_json)
        .await
        .map_err(Into::into)?;

    Ok(())
}

#[post("/insert-node-batch", data = "<input_data>")]
async fn insert_node_batch(
    conn: Connection<'_, Db>,
    input_data: Json<Vec<NodeJson>>,
) -> Result<(), ErrorResponder> {
    let db = conn.into_inner();
    let node_jsons = input_data.clone();

    try_join_all(
        node_jsons
            .into_iter()
            .map(|node_json| Mutate::insert_node(db, node_json)),
    )
    .await
    .map_err(Into::into)?;

    Ok(())
}

#[post("/insert-edge", data = "<input_data>")]
async fn insert_edge(
    conn: Connection<'_, Db>,
    input_data: Json<EdgeJson>,
) -> Result<(), ErrorResponder> {
    let db = conn.into_inner();
    let edge_json = input_data.clone();

    Mutate::insert_edge(db, edge_json)
        .await
        .map_err(Into::into)?;

    Ok(())
}

#[post("/insert-edge-batch", data = "<input_data>")]
async fn insert_edge_batch(
    conn: Connection<'_, Db>,
    input_data: Json<Vec<EdgeJson>>,
) -> Result<(), ErrorResponder> {
    let db = conn.into_inner();
    let edge_jsons = input_data.clone();

    try_join_all(
        edge_jsons
            .into_iter()
            .map(|edge_json| Mutate::insert_edge(db, edge_json)),
    )
    .await
    .map_err(Into::into)?;

    Ok(())
}

#[post("/clear-edge", data = "<input_data>")]
async fn clear_edge(
    conn: Connection<'_, Db>,
    input_data: Json<ClearEdgeJson>,
) -> Result<(), ErrorResponder> {
    let db = conn.into_inner();
    let clear_edge_json = input_data.clone();

    Mutate::clear_edge(db, clear_edge_json)
        .await
        .map_err(Into::into)?;

    Ok(())
}
