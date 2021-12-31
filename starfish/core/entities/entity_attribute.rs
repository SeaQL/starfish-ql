use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum Datatype {
    #[sea_orm(string_value = "Int")]
    Int,
    #[sea_orm(string_value = "String")]
    String,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "entity_attribute")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub entity_id: i32,
    pub name: String,
    pub datatype: Datatype,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::entity::Entity",
        from = "Column::EntityId",
        to = "super::entity::Column::Id"
    )]
    Entity,
}

impl Related<super::entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Entity.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
