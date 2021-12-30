mod common;

use common::{schema::create_tables, TestContext};
use sea_orm::{DbConn, DbErr};
use starfish::{
    core::entities::entity_attribute::Datatype,
    mutate::{EdgeJson, Mutate, NodeJson},
    schema::{EntityAttrJson, EntityJson, RelationJson, Schema},
};
use std::collections::HashMap;

#[smol_potat::test]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("starfish_tests").await;
    let db = &ctx.db;

    create_tables(db).await?;

    test_create_entities(db).await?;
    test_create_relations(db).await?;
    test_insert_node(db).await?;
    test_delete_node(db).await?;
    test_insert_edge(db).await?;
    test_delete_edge(db).await?;

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

async fn test_insert_node(db: &DbConn) -> Result<(), DbErr> {
    Mutate::insert_node(
        db,
        NodeJson {
            of: "crate".to_owned(),
            name: "sqlx".to_owned(),
            attributes: HashMap::from([
                ("version".into(), "0.5.10".into()),
                ("some_other_random_attr".into(), "xxx".into()),
            ]),
        },
    )
    .await?;

    Mutate::insert_node(
        db,
        NodeJson {
            of: "crate".to_owned(),
            name: "sea-orm".to_owned(),
            attributes: HashMap::from([
                ("version".into(), "0.5.0-rc.1".into()),
                ("some_other_random_attr".into(), "xxx".into()),
            ]),
        },
    )
    .await?;

    Mutate::insert_node(
        db,
        NodeJson {
            of: "crate".to_owned(),
            name: "sea-query".to_owned(),
            attributes: HashMap::from([
                ("version".into(), "0.20.0".into()),
                ("some_other_random_attr".into(), "xxx".into()),
            ]),
        },
    )
    .await?;

    Mutate::insert_node(
        db,
        NodeJson {
            of: "crate".to_owned(),
            name: "sea-schema".to_owned(),
            attributes: HashMap::from([
                ("version".into(), "0.4.0".into()),
                ("some_other_random_attr".into(), "xxx".into()),
            ]),
        },
    )
    .await?;

    Ok(())
}

async fn test_delete_node(db: &DbConn) -> Result<(), DbErr> {
    Mutate::delete_node(db, "crate".to_owned(), "sqlx".to_owned()).await?;

    Ok(())
}

async fn test_insert_edge(db: &DbConn) -> Result<(), DbErr> {
    Mutate::insert_edge(
        db,
        EdgeJson {
            name: "depends".to_owned(),
            from_node: "sea-orm".to_owned(),
            to_node: "sea-schema".to_owned(),
        },
    )
    .await?;

    Mutate::insert_edge(
        db,
        EdgeJson {
            name: "depends".to_owned(),
            from_node: "sea-orm".to_owned(),
            to_node: "sea-query".to_owned(),
        },
    )
    .await?;

    Mutate::insert_edge(
        db,
        EdgeJson {
            name: "depends".to_owned(),
            from_node: "sea-schema".to_owned(),
            to_node: "sea-query".to_owned(),
        },
    )
    .await?;

    Ok(())
}

async fn test_delete_edge(db: &DbConn) -> Result<(), DbErr> {
    Mutate::delete_edge(
        db,
        EdgeJson {
            name: "depends".to_owned(),
            from_node: "sea-orm".to_owned(),
            to_node: "sea-schema".to_owned(),
        },
    )
    .await?;

    Ok(())
}
