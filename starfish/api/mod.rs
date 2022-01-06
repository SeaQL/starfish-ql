#![allow(missing_docs)]

pub mod db;
mod handler;

use db::{pool, schema};
use rocket::fairing::{self, AdHoc};
use rocket::serde::json::{json, Value};
use rocket::{catch, catchers, Build, Responder, Rocket};
use sea_orm::DbErr;
use sea_orm_rocket::Database;

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &pool::Db::fetch(&rocket).unwrap().conn;
    let _ = schema::create_tables(conn).await;
    Ok(rocket)
}

pub fn rocket() -> Rocket<Build> {
    use figment::{
        providers::{Env, Format, Toml},
        Figment,
    };

    let figment = Figment::new()
        .merge(rocket::Config::default())
        .merge(Toml::file("Rocket.toml").nested())
        .merge(Env::prefixed("ROCKET_APP_").split("_"));

    rocket::custom(figment)
        .attach(pool::Db::init())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .mount("/schema", handler::schema::routes())
        .mount("/mutate", handler::mutate::routes())
        .mount("/query", handler::query::routes())
        .mount("/util", handler::util::routes())
        .register("/", catchers![not_found])
}

#[catch(404)]
fn not_found() -> Value {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

#[derive(Responder)]
#[response(status = 500, content_type = "json")]
struct ErrorResponder {
    message: String,
}

#[allow(clippy::from_over_into)]
impl Into<ErrorResponder> for DbErr {
    fn into(self) -> ErrorResponder {
        ErrorResponder {
            message: self.to_string(),
        }
    }
}
