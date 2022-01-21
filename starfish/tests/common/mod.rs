pub mod setup;

use sea_orm::DbConn;
use starfish_core::sea_orm;

pub struct TestContext {
    pub db: DbConn,
}

impl TestContext {
    pub async fn new(test_name: &str) -> Self {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_test_writer()
            .try_init();

        let base_url =
            std::env::var("DATABASE_URL").expect("Enviroment variable 'DATABASE_URL' not set");
        let db: DbConn = setup::setup(&base_url, test_name).await;

        Self { db }
    }
}
