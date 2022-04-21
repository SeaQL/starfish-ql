use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};
use starfish_core::sea_orm;

pub async fn setup(base_url: &str, db_name: &str) -> DatabaseConnection {
    let db = if cfg!(feature = "sqlx-mysql") {
        let url = format!("{}/mysql", base_url);
        let db = Database::connect(&url).await.unwrap();
        let _drop_db_result = db
            .execute(Statement::from_string(
                DbBackend::MySql,
                format!("DROP DATABASE IF EXISTS `{}`;", db_name),
            ))
            .await;

        let _create_db_result = db
            .execute(Statement::from_string(
                DbBackend::MySql,
                format!("CREATE DATABASE `{}`;", db_name),
            ))
            .await;

        let url = format!("{}/{}", base_url, db_name);
        Database::connect(&url).await.unwrap()
    } else if cfg!(feature = "sqlx-postgres") {
        let url = format!("{}/postgres", base_url);
        let db = Database::connect(&url).await.unwrap();
        let _drop_db_result = db
            .execute(Statement::from_string(
                DbBackend::Postgres,
                format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
            ))
            .await;

        let _create_db_result = db
            .execute(Statement::from_string(
                DbBackend::Postgres,
                format!("CREATE DATABASE \"{}\";", db_name),
            ))
            .await;

        let url = format!("{}/{}", base_url, db_name);
        Database::connect(&url).await.unwrap()
    } else {
        Database::connect(base_url).await.unwrap()
    };

    db
}
