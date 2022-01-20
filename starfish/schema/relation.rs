//! Define relation schema

use super::{format_node_table_name, Schema};
use crate::core::{entities::relation, lang::RelationJson};
use sea_orm::{ActiveModelTrait, ConnectionTrait, DbConn, DbErr, DeriveIden, Set};
use sea_query::{Alias, ColumnDef, ForeignKey, Index, Table};

impl Schema {
    /// Insert metadata of relation into database and create a corresponding node table
    pub async fn create_relation(db: &DbConn, relation_json: RelationJson) -> Result<(), DbErr> {
        relation::ActiveModel {
            name: Set(relation_json.name.clone()),
            from_entity: Set(relation_json.from_entity.clone()),
            to_entity: Set(relation_json.to_entity.clone()),
            directed: Set(relation_json.directed),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Self::create_link_table(
            db,
            &relation_json,
            format_node_table_name(&relation_json.from_entity),
            format_node_table_name(&relation_json.to_entity),
        )
        .await
    }

    async fn create_link_table(
        db: &DbConn,
        relation_json: &RelationJson,
        from_entity: String,
        to_entity: String,
    ) -> Result<(), DbErr> {
        let table = Alias::new(relation_json.get_table_name().as_str());
        let mut stmt = Table::create();
        stmt.table(table.clone())
            .col(
                ColumnDef::new(Alias::new("id"))
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Alias::new("from_node")).string().not_null())
            .col(ColumnDef::new(Alias::new("to_node")).string().not_null())
            .index(
                Index::create()
                    .name(&format!("idx-{}-{}", table.to_string(), "from_node"))
                    .table(table.clone())
                    .col(Alias::new("from_node")),
            )
            .index(
                Index::create()
                    .name(&format!("idx-{}-{}", table.to_string(), "to_node"))
                    .table(table.clone())
                    .col(Alias::new("to_node")),
            )
            .index(
                Index::create()
                    .unique()
                    .name(&format!(
                        "idx-{}-from_node-to_node",
                        relation_json.get_table_name()
                    ))
                    .col(Alias::new("from_node"))
                    .col(Alias::new("to_node")),
            )
            .foreign_key(
                ForeignKey::create()
                    .name(&format!(
                        "fk-{}-from-{}",
                        relation_json.get_table_name(),
                        from_entity
                    ))
                    .from_tbl(Alias::new(relation_json.get_table_name().as_str()))
                    .from_col(Alias::new("from_node"))
                    .to_tbl(Alias::new(from_entity.as_str()))
                    .to_col(Alias::new("name")),
            )
            .foreign_key(
                ForeignKey::create()
                    .name(&format!(
                        "fk-{}-to-{}",
                        relation_json.get_table_name(),
                        to_entity
                    ))
                    .from_tbl(Alias::new(relation_json.get_table_name().as_str()))
                    .from_col(Alias::new("to_node"))
                    .to_tbl(Alias::new(to_entity.as_str()))
                    .to_col(Alias::new("name")),
            );

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }
}
