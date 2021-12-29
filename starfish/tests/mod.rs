mod common;

use common::{schema::create_tables, TestContext};
use sea_orm::{DbConn, DbErr};
use starfish::{
    core::entities::entity_attribute::Datatype,
    schema::{EntityAttrJson, EntityJson, RelationJson, Schema},
};

#[smol_potat::test]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("starfish_tests").await;
    let db = &ctx.db;

    create_tables(db).await?;

    test_create_entities(db).await?;
    test_create_relations(db).await?;

    Ok(())
}

async fn test_create_entities(db: &DbConn) -> Result<(), DbErr> {
    let entity_json = EntityJson {
        name: "crate".to_owned(),
        attributes: vec![EntityAttrJson {
            name: "version".to_owned(),
            datatype: Datatype::String,
        }],
    };

    Schema::create_entity(db, entity_json).await?;

    Ok(())
}

async fn test_create_relations(db: &DbConn) -> Result<(), DbErr> {
    let relation_json = RelationJson {
        name: "depends".to_owned(),
        from_entity: "crate".to_owned(),
        to_entity: "crate".to_owned(),
        directed: true,
    };

    Schema::create_relation(db, relation_json).await?;

    Ok(())
}
