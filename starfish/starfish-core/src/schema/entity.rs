//! Define entity schema

use super::Schema;
use crate::{
    entities::{
        entity,
        entity_attribute::{self, Datatype},
    },
    lang::{EntityJson, iden::NodeIden},
};
use sea_orm::{ActiveModelTrait, ConnectionTrait, DbConn, DbErr, DeriveIden, Set};
use sea_query::{Alias, ColumnDef, Index, Table};

impl Schema {
    /// Insert entity metadata into database and create a corresponding node table
    pub async fn create_entity(db: &DbConn, entity_json: EntityJson) -> Result<(), DbErr> {
        let entity_json_bak = entity_json.clone();

        let entity = entity::ActiveModel {
            name: Set(entity_json.name),
            ..Default::default()
        }
        .insert(db)
        .await?;

        for attribute in entity_json.attributes.into_iter() {
            entity_attribute::ActiveModel {
                entity_id: Set(entity.id),
                name: Set(attribute.name),
                datatype: Set(attribute.datatype),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }

        Self::create_node_table(db, entity_json_bak).await
    }

    async fn create_node_table(db: &DbConn, entity_json: EntityJson) -> Result<(), DbErr> {
        let table = Alias::new(entity_json.get_table_name().as_str());
        let mut stmt = Table::create();
        stmt.table(table.clone())
            .col(
                ColumnDef::new(NodeIden::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(NodeIden::Name)
                    .string()
                    .not_null()
                    .unique_key(),
            )
            .index(
                Index::create()
                    .name(&format!("idx-{}-{}", table.to_string(), "name"))
                    .table(table.clone())
                    .col(NodeIden::Name),
            );

        for attribute in entity_json.attributes.into_iter() {
            let mut column_def = ColumnDef::new(Alias::new(attribute.get_column_name().as_str()));
            match attribute.datatype {
                Datatype::Int => column_def.integer(),
                Datatype::String => column_def.string(),
            };
            stmt.col(&mut column_def);
        }

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }
}
