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

impl Datatype {
    pub fn value_with_datatype(&self, value: Option<&serde_json::Value>) -> Value {
        match self {
            Datatype::Int => {
                if let Some(value) = value {
                    value.as_i64().into()
                } else {
                    None::<i64>.into()
                }
            }
            Datatype::String => {
                if let Some(value) = value {
                    value.as_str().into()
                } else {
                    None::<String>.into()
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "entity_attribute")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(indexed)]
    pub entity_id: i32,
    #[sea_orm(indexed)]
    pub name: String,
    #[sea_orm(indexed)]
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
