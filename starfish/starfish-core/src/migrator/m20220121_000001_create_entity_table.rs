use sea_schema::{migration::*, sea_query::*};
use crate::entities::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220121_000001_create_entity_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use entity::*;

        let stmt = Table::create()
            .if_not_exists()
            .table(Entity)
            .col(
                ColumnDef::new(Column::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(Column::Name)
                    .string()
                    .not_null()
                    .unique_key(),
            )
            .index(
                Index::create()
                    .name(&format!(
                        "idx-{}-{}",
                        Entity.to_string(),
                        Column::Name.to_string()
                    ))
                    .table(Entity)
                    .col(Column::Name),
            )
            .to_owned();

        manager.create_table(stmt).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use entity::*;

        manager
            .drop_table(Table::drop().table(Entity).to_owned())
            .await
    }
}
