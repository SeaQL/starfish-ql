use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "relation")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique, indexed)]
    pub name: String,
    #[sea_orm(indexed)]
    pub from_entity: String,
    #[sea_orm(indexed)]
    pub to_entity: String,
    #[sea_orm(indexed)]
    pub directed: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::entity::Entity",
        from = "Column::FromEntity",
        to = "super::entity::Column::Name"
    )]
    FromEntity,
    #[sea_orm(
        belongs_to = "super::entity::Entity",
        from = "Column::ToEntity",
        to = "super::entity::Column::Name"
    )]
    ToEntity,
}

impl ActiveModelBehavior for ActiveModel {}
