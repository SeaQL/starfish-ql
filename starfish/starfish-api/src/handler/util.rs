use crate::{check_auth_match, db::pool::Db, ErrorResponder};
use migration::{Migrator, MigratorTrait};
use rocket::{get, routes};
use sea_orm_rocket::Connection;

pub fn routes() -> Vec<rocket::Route> {
    routes![reset]
}

#[get("/reset?<auth>")]
async fn reset(conn: Connection<'_, Db>, auth: Option<String>) -> Result<(), ErrorResponder> {
    check_auth_match(auth)?;

    let db = conn.into_inner();

    Migrator::fresh(db).await.map_err(Into::into)?;

    Ok(())
}
