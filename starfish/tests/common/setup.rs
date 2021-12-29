use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

pub async fn setup(base_url: &str, db_name: &str) -> DatabaseConnection {
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
}
