mod common;

use std::collections::HashMap;

use common::TestContext;
use migration::{Migrator, MigratorTrait, SchemaManager};
use sea_orm::DbErr;
use starfish_core::lang::{
    Edge, EdgeJsonBatch, MutateInsertJson, MutateJson, Node, NodeJsonBatch, MutateUpdateJson, MutateNodeSelectorJson, MutateEdgeSelectorJson, MutateEdgeContentJson,
};
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
    assert!(
        schema_manager
            .has_column("node_crate", "attr_version")
            .await?
    );
    assert!(schema_manager.has_table("edge_depends").await?);
    assert!(
        schema_manager
            .has_column("edge_depends", "from_node")
            .await?
    );
    assert!(schema_manager.has_column("edge_depends", "to_node").await?);

    println!("# Schema defined successfully! #");

    let mutate_json = MutateJson::insert(MutateInsertJson::node(NodeJsonBatch {
        of: "crate".to_owned(),
        nodes: vec![Node {
            name: "sea-orm".to_owned(),
            attributes: HashMap::from([("version".to_owned(), "1.0".into())]),
        }],
    }));

    Mutate::mutate(db, mutate_json, false).await?;
    // New node should be inserted (name: "sea-orm", version: "1.0")

    let mutate_json = MutateJson::insert(MutateInsertJson::node(NodeJsonBatch {
        of: "crate".to_owned(),
        nodes: vec![
            Node {
                name: "sea-orm".to_owned(),
                attributes: HashMap::from([("version".to_owned(), "2.0".into())]),
            },
            Node {
                name: "sea-query".to_owned(),
                attributes: HashMap::from([("version".to_owned(), "1.0".into())]),
            },
        ],
    }));

    assert!(Mutate::mutate(db, mutate_json.clone(), false)
        .await
        .is_err());
    Mutate::mutate(db, mutate_json, true).await?;
    // New node should be inserted (name: "sea-query", version: "1.0")
    // Node should be updated (name: "sea-orm", version: "2.0")

    let mutate_json = MutateJson::insert(MutateInsertJson::edge(EdgeJsonBatch {
        of: "depends".to_owned(),
        edges: vec![Edge {
            from_node: "sea-orm".to_owned(),
            to_node: "sea-query".to_owned(),
        }],
    }));

    Mutate::mutate(db, mutate_json.clone(), false).await?;
    // New edge should be inserted (from_node: "sea-orm", to_node: "sea-query")
    Mutate::mutate(db, mutate_json, false).await?;
    // NO new edge should be inserted
    
    println!("# Node and edges inserted successfully! #");

    let mutate_json = MutateJson::update(MutateUpdateJson::node {
        selector: MutateNodeSelectorJson { of: "crate".to_owned(), name: None, attributes: HashMap::from([("version".to_owned(), "2.0".into())]) },
        content: HashMap::from([("version".to_owned(), "3.14".into())])
    });

    Mutate::mutate(db, mutate_json, false).await?;
    // Node should be updated (name: "sea-orm", version: "3.14")

    let mutate_json = MutateJson::update(MutateUpdateJson::edge {
        selector: MutateEdgeSelectorJson {
            of: "crate".to_owned(),
            edge_content: MutateEdgeContentJson { from_node: Some("sea-orm".to_owned()), to_node: None }
        },
        content: MutateEdgeContentJson { from_node: Some("sea-query".to_owned()), to_node: Some("sea-orm".to_owned()) }
    });

    Mutate::mutate(db, mutate_json, false).await?;
    // Edge should be updated (from_node: "sea-query", to_node: "sea-orm")
    // There should still be only 1 edge

    println!("# Node and edges updated successfully! #");

    Ok(())
}
