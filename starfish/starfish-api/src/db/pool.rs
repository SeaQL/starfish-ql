use async_trait::async_trait;
use sea_orm::{ConnectOptions, DbConn};
use sea_orm_rocket::{rocket::figment::Figment, Config, Database};
use starfish_core::sea_orm;
use std::env;
use std::time::Duration;

#[derive(Database, Debug)]
#[database("starfish")]
pub struct Db(RocketDbPool);

#[derive(Debug)]
pub struct RocketDbPool {
    pub conn: DbConn,
}

#[async_trait]
impl sea_orm_rocket::Pool for RocketDbPool {
    type Error = sea_orm::DbErr;

    type Connection = DbConn;

    async fn init(figment: &Figment) -> Result<Self, Self::Error> {
        let config = figment.extract::<Config>().unwrap();
        // Extract `DATABASE_URL` defined in environment variable (if any)
        // Otherwise, use `DATABASE_URL`defined in `Rocket.toml`
        let mut options: ConnectOptions = env::var("DATABASE_URL")
            .map(Into::into)
            .unwrap_or_else(|_| config.url.into());
        options = dbg!(options);
        options
            .max_connections(config.max_connections as u32)
            .min_connections(config.min_connections.unwrap_or_default())
            .connect_timeout(Duration::from_secs(config.connect_timeout));
        if let Some(idle_timeout) = config.idle_timeout {
            options.idle_timeout(Duration::from_secs(idle_timeout));
        }
        let conn = sea_orm::Database::connect(options).await?;

        Ok(RocketDbPool { conn })
    }

    fn borrow(&self) -> &Self::Connection {
        &self.conn
    }
}
