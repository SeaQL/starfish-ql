use crate::api::db::{pool::Db, schema::create_tables};
use crate::api::ErrorResponder;
use rocket::{get, routes};
use sea_orm::{ConnectionTrait, Statement};
use sea_orm_rocket::Connection;

pub fn routes() -> Vec<rocket::Route> {
    routes![reset]
}

#[get("/reset")]
async fn reset(conn: Connection<'_, Db>) -> Result<(), ErrorResponder> {
    let db = conn.into_inner();

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "DROP DATABASE starfish".to_owned(),
    ))
    .await
    .map_err(Into::into)?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE DATABASE starfish".to_owned(),
    ))
    .await
    .map_err(Into::into)?;

    create_tables(db).await.map_err(Into::into)?;

    Ok(())
}
