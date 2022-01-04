use crate::api::db::{pool::Db, schema::create_tables};
use crate::api::ErrorResponder;
use crate::core::entities::{entity, relation};
use crate::schema::{format_edge_table_name, format_node_table_name};
use rocket::{get, routes};
use sea_orm::{ConnectionTrait, EntityTrait, Statement};
use sea_orm_rocket::Connection;

pub fn routes() -> Vec<rocket::Route> {
    routes![reset]
}

#[get("/reset")]
async fn reset(conn: Connection<'_, Db>) -> Result<(), ErrorResponder> {
    let db = conn.into_inner();
    let mut tables = vec![
        "entity".to_owned(),
        "entity_attribute".to_owned(),
        "relation".to_owned(),
    ];

    for entity in entity::Entity::find()
        .all(db)
        .await
        .map_err(Into::into)?
        .into_iter()
    {
        tables.push(format_node_table_name(entity.name));
    }

    for relation in relation::Entity::find()
        .all(db)
        .await
        .map_err(Into::into)?
        .into_iter()
    {
        tables.push(format_edge_table_name(relation.name));
    }

    for table in tables.into_iter().rev() {
        db.execute(Statement::from_string(
            db.get_database_backend(),
            format!("DROP TABLE IF EXISTS {} CASCADE", table),
        ))
        .await
        .map_err(Into::into)?;
    }

    create_tables(db).await.map_err(Into::into)?;

    Ok(())
}
