#![allow(missing_docs)]

pub mod db;
mod handler;

use async_trait::async_trait;
use db::pool;
use migration::{Migrator, MigratorTrait};
use rocket::fairing::{self, AdHoc, Fairing, Info, Kind};
use rocket::http::Header;
use rocket::serde::json::{json, Value};
use rocket::tokio::runtime;
use rocket::{catch, catchers, Build, Request, Responder, Response, Rocket};
use sea_orm::DbErr;
use sea_orm_rocket::Database;
use starfish_core::sea_orm;

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &pool::Db::fetch(&rocket).unwrap().conn;
    Migrator::up(conn, None).await.unwrap();
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
        .attach(Cors)
        .mount("/schema", handler::schema::routes())
        .mount("/mutate", handler::mutate::routes())
        .mount("/query", handler::query::routes())
        .mount("/util", handler::util::routes())
        .mount("/", handler::core::routes())
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

#[allow(clippy::from_over_into)]
impl Into<ErrorResponder> for &str {
    fn into(self) -> ErrorResponder {
        ErrorResponder {
            message: self.to_string(),
        }
    }
}

fn check_auth_match(auth: Option<String>) -> Result<(), ErrorResponder> {
    let err = Err("Authorization failed.".into());
    match (auth, std::env::var("API_AUTH_KEY").ok()) {
        (Some(auth), Some(expected)) => {
            if !auth.eq(&expected) {
                err
            } else {
                Ok(())
            }
        }
        (None, Some(_)) => err,
        (_, None) => Ok(()),
    }
}

#[derive(Debug)]
pub struct Cors;

#[async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Cross-Origin-Resource-Sharing Middleware",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("access-control-allow-origin", "*"));
        response.set_header(Header::new(
            "access-control-allow-methods",
            "GET, POST, PATCH, OPTIONS",
        ));
    }
}

pub fn main() {
    runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(rocket().launch())
        .unwrap();
}
