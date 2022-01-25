use crate::db::pool::Db;
use crate::ErrorResponder;
use migration::sea_orm::EntityTrait;
use rocket::serde::json::Json;
use rocket::{post, routes};
use sea_orm_rocket::Connection;
use starfish_core::entities;
use starfish_core::lang::{ClearEdgeJson, EdgeJson, EdgeJsonBatch, NodeJson, NodeJsonBatch};
use starfish_core::mutate::Mutate;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        insert_node,
        insert_node_batch,
        insert_edge,
        insert_edge_batch,
        clear_edge,
        cal_conn,
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
    input_data: Json<NodeJsonBatch>,
) -> Result<(), ErrorResponder> {
    let db = conn.into_inner();
    let node_json_batch = input_data.clone();

    Mutate::insert_node_batch(db, node_json_batch, true)
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
    input_data: Json<EdgeJsonBatch>,
) -> Result<(), ErrorResponder> {
    let db = conn.into_inner();
    let edge_json_batch = input_data.clone();

    Mutate::insert_edge_batch(db, edge_json_batch)
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

#[post("/cal-conn")]
async fn cal_conn(conn: Connection<'_, Db>) -> Result<(), ErrorResponder> {
    let db = conn.into_inner();

    let relations = entities::Relation::find()
        .all(db)
        .await
        .map_err(Into::into)?;

    for relation in relations.into_iter() {
        let relation_name = relation.name.as_str();
        let from_node = relation.from_entity.as_str();
        let to_node = relation.to_entity.as_str();

        Mutate::calculate_simple_connectivity(db, relation_name, from_node, to_node)
            .await
            .map_err(Into::into)?;

        Mutate::calculate_compound_connectivity(db, relation_name, to_node)
            .await
            .map_err(Into::into)?;

        for (weight, col_name) in [
            (0.3, "in_conn_complex03"),
            (0.5, "in_conn_complex05"),
            (0.7, "in_conn_complex07"),
        ] {
            Mutate::calculate_complex_connectivity(
                db,
                relation_name,
                to_node,
                weight,
                f64::EPSILON,
                col_name,
            )
            .await
            .map_err(Into::into)?;
        }
    }

    Ok(())
}
