use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "relation")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
    pub from_entity_id: i32,
    pub to_entity_id: i32,
    pub directed: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::entity::Entity",
        from = "Column::FromEntityId",
        to = "super::entity::Column::Id"
    )]
    FromEntity,
    #[sea_orm(
        belongs_to = "super::entity::Entity",
        from = "Column::ToEntityId",
        to = "super::entity::Column::Id"
    )]
    ToEntity,
}

impl ActiveModelBehavior for ActiveModel {}
