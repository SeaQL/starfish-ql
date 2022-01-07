use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "entity")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique, indexed)]
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::entity_attribute::Entity")]
    EntityAttribute,
}

impl Related<super::entity_attribute::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EntityAttribute.def()
    }
}

#[derive(Debug)]
pub struct DependencyLink;

impl Linked for DependencyLink {
    type FromEntity = Entity;

    type ToEntity = Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::relation::Relation::FromEntity.def(),
            super::relation::Relation::ToEntity.def(),
        ]
    }
}

impl ActiveModelBehavior for ActiveModel {}
