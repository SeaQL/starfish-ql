mod common;

use std::collections::HashMap;

use common::TestContext;
use migration::{Migrator, MigratorTrait, SchemaManager};
use sea_orm::DbErr;
use starfish_core::lang::{MutateJson, MutateInsertContentJson, NodeJsonBatch, Node, EdgeJsonBatch, Edge};
use starfish_core::mutate::Mutate;
use starfish_core::sea_orm;
use starfish_core::{
    entities::entity_attribute::Datatype,
    lang::{EntityAttrJson, EntityJson, RelationJson, SchemaDefineJson, SchemaJson},
    schema::Schema,
};

#[smol_potat::test]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("starfish_core").await;
    let db = &ctx.db;

    Migrator::fresh(db).await?;

    let schema_json = SchemaJson {
        define: SchemaDefineJson {
            entities: vec![EntityJson {
                name: "crate".to_owned(),
                attributes: vec![EntityAttrJson {
                    name: "version".to_owned(),
                    datatype: Datatype::String,
                }],
            }],
            relations: vec![RelationJson {
                name: "depends".to_owned(),
                from_entity: "crate".to_owned(),
                to_entity: "crate".to_owned(),
                directed: true,
            }],
        },
    };

    Schema::define_schema(db, schema_json).await?;

    let schema_manager = SchemaManager::new(db);
    assert!(schema_manager.has_table("node_crate").await?);
    assert!(schema_manager.has_column("node_crate", "attr_version").await?);
    assert!(schema_manager.has_table("edge_depends").await?);
    assert!(schema_manager.has_column("edge_depends", "from_node").await?);
    assert!(schema_manager.has_column("edge_depends", "to_node").await?);

    let mutate_json = MutateJson::insert(
        MutateInsertContentJson::node(
            NodeJsonBatch {
                of: "crate".to_owned(),
                nodes: vec![
                    Node { name: "sea-orm".to_owned(), attributes: HashMap::from([("version".to_owned(), "1.0".into())]) }
                ]
            }
        )
    );

    Mutate::mutate(db, mutate_json, false).await?;

    let mutate_json = MutateJson::insert(
        MutateInsertContentJson::node(
            NodeJsonBatch {
                of: "crate".to_owned(),
                nodes: vec![
                    Node { name: "sea-orm".to_owned(), attributes: HashMap::from([("version".to_owned(), "2.0".into())]) },
                    Node { name: "sea-query".to_owned(), attributes: HashMap::from([("version".to_owned(), "1.0".into())]) },
                ]
            }
        )
    );

    assert!(Mutate::mutate(db, mutate_json.clone(), false).await.is_err());
    Mutate::mutate(db, mutate_json, true).await?;

    let mutate_json = MutateJson::insert(
        MutateInsertContentJson::edge(
            EdgeJsonBatch {
                of: "depends".to_owned(),
                edges: vec![
                    Edge { from_node: "sea-orm".to_owned(), to_node: "sea-query".to_owned() },
                ]
            }
        )
    );

    Mutate::mutate(db, mutate_json, false).await?;

    Ok(())
}