use crate::core::entities::{self, *};
use sea_orm::{
    error::*, sea_query, ConnectionTrait, DatabaseConnection, DbBackend, DbConn, EntityTrait,
    ExecResult, Schema,
};
use sea_query::{ColumnDef, ForeignKeyCreateStatement, Table, TableCreateStatement};

#[cfg(test)]
use pretty_assertions::assert_eq;

pub async fn create_tables(db: &DatabaseConnection) -> Result<(), DbErr> {
    create_entity_table(db).await?;
    create_relation_table(db).await?;
    create_entity_attribute_table(db).await?;

    Ok(())
}

pub async fn create_entity_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    use entities::entity::*;

    let stmt = sea_query::Table::create()
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
        .to_owned();

    create_table(db, &stmt, Entity).await
}

pub async fn create_relation_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    use entities::relation::*;

    let stmt = sea_query::Table::create()
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
        .col(ColumnDef::new(Column::FromEntityId).integer().not_null())
        .col(ColumnDef::new(Column::ToEntityId).integer().not_null())
        .col(ColumnDef::new(Column::Directed).boolean().not_null())
        .foreign_key(
            ForeignKeyCreateStatement::new()
                .name("fk-relation-entity-1")
                .from_tbl(Entity)
                .from_col(Column::FromEntityId)
                .to_tbl(entity::Entity)
                .to_col(entity::Column::Id),
        )
        .foreign_key(
            ForeignKeyCreateStatement::new()
                .name("fk-relation-entity-2")
                .from_tbl(Entity)
                .from_col(Column::ToEntityId)
                .to_tbl(entity::Entity)
                .to_col(entity::Column::Id),
        )
        .to_owned();

    // FIXME: https://github.com/SeaQL/sea-orm/issues/405
    // create_table(db, &stmt, Entity).await
    create_table_without_asserts(db, &stmt).await
}

pub async fn create_entity_attribute_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    use entities::entity_attribute::*;

    let stmt = sea_query::Table::create()
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

    create_table(db, &stmt, Entity).await
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
