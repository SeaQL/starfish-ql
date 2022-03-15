use crate::db::pool::Db;
use crate::ErrorResponder;
use rocket::{get, routes};
use sea_orm_rocket::Connection;
use starfish_core::migrator::{Migrator, MigratorTrait};

pub fn routes() -> Vec<rocket::Route> {
    routes![reset]
}

#[get("/reset")]
async fn reset(conn: Connection<'_, Db>) -> Result<(), ErrorResponder> {
    let db = conn.into_inner();

    Migrator::fresh(db).await.map_err(Into::into)?;

    Ok(())
}
