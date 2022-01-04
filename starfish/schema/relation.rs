//! Define relation schema

use super::{format_edge_table_name, format_node_table_name, Schema};
use crate::core::entities::{entity, relation};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbConn, DbErr, EntityTrait, QueryFilter, Set,
};
use sea_query::{Alias, ColumnDef, ForeignKey, Index, Table};
use serde::{Deserialize, Serialize};

/// Metadata of relation, deserialized as struct from json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationJson {
    /// Name of relation
    pub name: String,
    /// Name of related entity (from side)
    pub from_entity: String,
    /// Name of related entity (to side)
    pub to_entity: String,
    /// Directed relation
    pub directed: bool,
}

impl RelationJson {
    /// Prefix the name of relation table
    pub fn get_table_name(&self) -> String {
        format_edge_table_name(&self.name)
    }
}

impl Schema {
    /// Insert metadata of relation into database and create a corresponding node table
    pub async fn create_relation(db: &DbConn, relation_json: RelationJson) -> Result<(), DbErr> {
        let relation_json_bak = relation_json.clone();

        let from_entity = entity::Entity::find()
            .filter(entity::Column::Name.eq(relation_json.from_entity.as_str()))
            .one(db)
            .await?
            .ok_or_else(|| {
                DbErr::Custom(format!(
                    "Entity of name '{}' could not be found",
                    relation_json.from_entity
                ))
            })?;

        let to_entity = entity::Entity::find()
            .filter(entity::Column::Name.eq(relation_json.to_entity.as_str()))
            .one(db)
            .await?
            .ok_or_else(|| {
                DbErr::Custom(format!(
                    "Entity of name '{}' could not be found",
                    relation_json.to_entity
                ))
            })?;

        relation::ActiveModel {
            name: Set(relation_json.name),
            from_entity_id: Set(from_entity.id),
            to_entity_id: Set(to_entity.id),
            directed: Set(relation_json.directed),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Self::create_link_table(
            db,
            relation_json_bak,
            format_node_table_name(from_entity.name),
            format_node_table_name(to_entity.name),
        )
        .await
    }

    async fn create_link_table(
        db: &DbConn,
        relation_json: RelationJson,
        from_entity: String,
        to_entity: String,
    ) -> Result<(), DbErr> {
        let mut stmt = Table::create();

        stmt.table(Alias::new(relation_json.get_table_name().as_str()))
            .col(
                ColumnDef::new(Alias::new("id"))
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(Alias::new("from_node_id"))
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(Alias::new("to_node_id"))
                    .integer()
                    .not_null(),
            )
            .index(
                Index::create()
                    .unique()
                    .name(&format!(
                        "idx-{}-from_node_id-to_node_id",
                        relation_json.get_table_name()
                    ))
                    .col(Alias::new("from_node_id"))
                    .col(Alias::new("to_node_id")),
            )
            .foreign_key(
                ForeignKey::create()
                    .name(&format!(
                        "fk-{}-from-{}",
                        relation_json.get_table_name(),
                        from_entity
                    ))
                    .from_tbl(Alias::new(relation_json.get_table_name().as_str()))
                    .from_col(Alias::new("from_node_id"))
                    .to_tbl(Alias::new(from_entity.as_str()))
                    .to_col(Alias::new("id")),
            )
            .foreign_key(
                ForeignKey::create()
                    .name(&format!(
                        "fk-{}-to-{}",
                        relation_json.get_table_name(),
                        to_entity
                    ))
                    .from_tbl(Alias::new(relation_json.get_table_name().as_str()))
                    .from_col(Alias::new("to_node_id"))
                    .to_tbl(Alias::new(to_entity.as_str()))
                    .to_col(Alias::new("id")),
            );

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }
}
