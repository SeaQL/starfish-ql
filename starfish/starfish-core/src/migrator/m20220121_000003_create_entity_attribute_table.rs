use crate::entities::*;
use sea_schema::{migration::*, sea_query::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220121_000003_create_entity_attribute_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use entity_attribute::*;

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
            .col(ColumnDef::new(Column::EntityId).integer().not_null())
            .col(ColumnDef::new(Column::Name).string().not_null())
            .col(ColumnDef::new(Column::Datatype).string().not_null())
            .foreign_key(
                ForeignKeyCreateStatement::new()
                    .name("fk-entity_attribute-entity")
                    .from_tbl(Entity)
                    .from_col(Column::EntityId)
                    .to_tbl(entity::Entity)
                    .to_col(entity::Column::Id),
            )
            .to_owned();

        manager.create_table(stmt).await?;

        manager
            .create_index(
                Index::create()
                    .name(&format!(
                        "idx-{}-{}",
                        Entity.to_string(),
                        Column::EntityId.to_string()
                    ))
                    .table(Entity)
                    .col(Column::EntityId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(&format!(
                        "idx-{}-{}",
                        Entity.to_string(),
                        Column::Name.to_string()
                    ))
                    .table(Entity)
                    .col(Column::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(&format!(
                        "idx-{}-{}",
                        Entity.to_string(),
                        Column::Datatype.to_string()
                    ))
                    .table(Entity)
                    .col(Column::Datatype)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use entity_attribute::*;

        manager
            .drop_table(Table::drop().table(Entity).to_owned())
            .await
    }
}
