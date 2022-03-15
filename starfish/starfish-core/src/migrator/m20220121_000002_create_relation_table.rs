use crate::entities::*;
use sea_schema::{migration::*, sea_query::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220121_000002_create_relation_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use relation::*;

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
            .col(ColumnDef::new(Column::FromEntity).string().not_null())
            .col(ColumnDef::new(Column::ToEntity).string().not_null())
            .col(ColumnDef::new(Column::Directed).boolean().not_null())
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
            .index(
                Index::create()
                    .name(&format!(
                        "idx-{}-{}",
                        Entity.to_string(),
                        Column::FromEntity.to_string()
                    ))
                    .table(Entity)
                    .col(Column::FromEntity),
            )
            .index(
                Index::create()
                    .name(&format!(
                        "idx-{}-{}",
                        Entity.to_string(),
                        Column::ToEntity.to_string()
                    ))
                    .table(Entity)
                    .col(Column::ToEntity),
            )
            .index(
                Index::create()
                    .name(&format!(
                        "idx-{}-{}",
                        Entity.to_string(),
                        Column::Directed.to_string()
                    ))
                    .table(Entity)
                    .col(Column::Directed),
            )
            .foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk-relation-from_entity")
                    .from_tbl(Entity)
                    .from_col(Column::FromEntity)
                    .to_tbl(entity::Entity)
                    .to_col(entity::Column::Name),
            )
            .foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk-relation-to_entity")
                    .from_tbl(Entity)
                    .from_col(Column::ToEntity)
                    .to_tbl(entity::Entity)
                    .to_col(entity::Column::Name),
            )
            .to_owned();

        manager.create_table(stmt).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use relation::*;

        manager
            .drop_table(Table::drop().table(Entity).to_owned())
            .await
    }
}
