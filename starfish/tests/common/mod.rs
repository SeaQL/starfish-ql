pub mod schema;
pub mod setup;

use pretty_assertions::assert_eq;
use sea_orm::{ConnectionTrait, DbBackend, DbConn, DbErr, EntityTrait, ExecResult, Schema};
use sea_query::{Table, TableCreateStatement};

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

pub async fn create_table<E>(
    db: &DbConn,
    create: &TableCreateStatement,
    entity: E,
) -> Result<ExecResult, DbErr>
where
    E: EntityTrait,
{
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    assert_eq!(
        builder.build(&schema.create_table_from_entity(entity)),
        builder.build(create)
    );

    create_table_without_asserts(db, create).await
}

pub async fn create_table_without_asserts(
    db: &DbConn,
    create: &TableCreateStatement,
) -> Result<ExecResult, DbErr> {
    let builder = db.get_database_backend();
    if builder != DbBackend::Sqlite {
        let stmt = builder.build(
            Table::drop()
                .table(create.get_table_name().unwrap().clone())
                .if_exists()
                .cascade(),
        );
        db.execute(stmt).await?;
    }
    db.execute(builder.build(create)).await
}
