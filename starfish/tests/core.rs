mod common;

use std::collections::HashMap;

use common::TestContext;
use migration::sea_orm::{ConnectionTrait, DbConn, FromQueryResult};
use migration::{Migrator, MigratorTrait, SchemaManager};
use sea_orm::DbErr;
use starfish_core::lang::mutate::{
    MutateDeleteJson, MutateEdgeContentJson, MutateEdgeSelectorJson, MutateInsertJson, MutateJson,
    MutateNodeSelectorJson, MutateUpdateJson,
};
use starfish_core::lang::schema::{SchemaDefineJson, SchemaJson};
use starfish_core::lang::{Edge, EdgeJsonBatch, Node, NodeJsonBatch};
use starfish_core::mutate::Mutate;
use starfish_core::schema::{format_edge_table_name, format_node_table_name};
use starfish_core::sea_orm;
use starfish_core::sea_query::{Alias, Cond, Expr, Query};
use starfish_core::{
    entities::entity_attribute::Datatype,
    lang::{EntityAttrJson, EntityJson, RelationJson},
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

    {
        // New node should be inserted (name: "sea-orm", version: "1.0")
        let sea_orm = TestNode::get_with_name(db, "crate", "sea-orm")
            .await?
            .unwrap();
        assert_eq!(sea_orm.name, "sea-orm");
        assert_eq!(sea_orm.attr_version, "1.0");
    }

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
    {
        // New node should be inserted (name: "sea-query", version: "1.0")
        // Node should be updated (name: "sea-orm", version: "2.0")
        let sea_query = TestNode::get_with_name(db, "crate", "sea-query")
            .await?
            .unwrap();
        assert_eq!(sea_query.name, "sea-query");
        assert_eq!(sea_query.attr_version, "1.0");
        let sea_orm = TestNode::get_with_name(db, "crate", "sea-orm")
            .await?
            .unwrap();
        assert_eq!(sea_orm.name, "sea-orm");
        assert_eq!(sea_orm.attr_version, "2.0");
    }

    let mutate_json = MutateJson::insert(MutateInsertJson::edge(EdgeJsonBatch {
        of: "depends".to_owned(),
        edges: vec![Edge {
            from_node: "sea-orm".to_owned(),
            to_node: "sea-query".to_owned(),
        }],
    }));

    Mutate::mutate(db, mutate_json.clone(), false).await?;
    {
        // New edge should be inserted (from_node: "sea-orm", to_node: "sea-query")
        assert!(
            TestEdge::get_with_from_to(db, "depends", "sea-orm", "sea-query")
                .await?
                .is_some()
        );
        assert!(
            TestEdge::get_with_from_to(db, "depends", "sea-query", "sea-orm")
                .await?
                .is_none()
        );
    }
    Mutate::mutate(db, mutate_json, false).await?;
    {
        // NO new edge should be inserted
        assert_eq!(TestEdge::get_all(db, "depends").await?.len(), 1);
    }

    println!("# Node and edges inserted successfully! #");

    let mutate_json = MutateJson::update(MutateUpdateJson::node {
        selector: MutateNodeSelectorJson {
            of: "crate".to_owned(),
            name: None,
            attributes: HashMap::from([("version".to_owned(), "2.0".into())]),
        },
        content: HashMap::from([("version".to_owned(), "3.14".into())]),
    });

    Mutate::mutate(db, mutate_json, false).await?;
    {
        // Node should be updated (name: "sea-orm", version: "3.14")
        let sea_orm = TestNode::get_with_name(db, "crate", "sea-orm")
            .await?
            .unwrap();
        assert_eq!(sea_orm.attr_version, "3.14");
    }

    let mutate_json = MutateJson::update(MutateUpdateJson::edge {
        selector: MutateEdgeSelectorJson {
            of: "depends".to_owned(),
            edge_content: MutateEdgeContentJson {
                from_node: Some("sea-orm".to_owned()),
                to_node: None,
            },
        },
        content: MutateEdgeContentJson {
            from_node: Some("sea-query".to_owned()),
            to_node: Some("sea-orm".to_owned()),
        },
    });

    Mutate::mutate(db, mutate_json, false).await?;
    {
        // Edge should be updated (from_node: "sea-query", to_node: "sea-orm")
        assert!(
            TestEdge::get_with_from_to(db, "depends", "sea-query", "sea-orm")
                .await?
                .is_some()
        );
        // There should still be only 1 edge
        assert_eq!(TestEdge::get_all(db, "depends").await?.len(), 1);
    }

    println!("# Node and edges updated successfully! #");

    let mutate_json = MutateJson::delete(MutateDeleteJson::edge(MutateEdgeSelectorJson {
        of: "depends".to_owned(),
        edge_content: MutateEdgeContentJson {
            from_node: Some("sea-query".to_owned()),
            to_node: None,
        },
    }));

    Mutate::mutate(db, mutate_json, false).await?;
    {
        // Edge should be deleted (from_node: "sea-query", to_node: "sea-orm")
        assert!(TestEdge::get_all(db, "depends").await?.is_empty());
    }

    let mutate_json = MutateJson::delete(MutateDeleteJson::node(MutateNodeSelectorJson {
        of: "crate".to_owned(),
        name: Some("sea-orm".to_owned()),
        attributes: HashMap::new(),
    }));

    Mutate::mutate(db, mutate_json, false).await?;
    {
        // Node should be deleted (name: "sea-orm")
        assert!(
            TestEdge::get_with_from_to(db, "depends", "sea-query", "sea-orm")
                .await?
                .is_none()
        );
    }

    println!("# Node and edges deleted successfully! #");

    Ok(())
}

#[derive(Debug, Clone, FromQueryResult)]
struct TestNode {
    name: String,
    attr_version: String,
}

impl TestNode {
    async fn get_with_name(db: &DbConn, of: &str, name: &str) -> Result<Option<Self>, DbErr> {
        let stmt = Query::select()
            .column(Alias::new("name"))
            .column(Alias::new("attr_version"))
            .from(Alias::new(&format_node_table_name(of)))
            .and_where(Expr::col(Alias::new("name")).eq(name))
            .to_owned();

        let builder = db.get_database_backend();

        Self::find_by_statement(builder.build(&stmt)).one(db).await
    }
}

#[derive(Debug, Clone, FromQueryResult)]
struct TestEdge {
    from_node: String,
    to_node: String,
}

impl TestEdge {
    async fn get_with_from_to(
        db: &DbConn,
        of: &str,
        from: &str,
        to: &str,
    ) -> Result<Option<Self>, DbErr> {
        let stmt = Query::select()
            .column(Alias::new("from_node"))
            .column(Alias::new("to_node"))
            .from(Alias::new(&format_edge_table_name(of)))
            .cond_where(
                Cond::all()
                    .add(Expr::col(Alias::new("from_node")).eq(from))
                    .add(Expr::col(Alias::new("to_node")).eq(to)),
            )
            .to_owned();

        let builder = db.get_database_backend();

        Self::find_by_statement(builder.build(&stmt)).one(db).await
    }

    async fn get_all(db: &DbConn, of: &str) -> Result<Vec<Self>, DbErr> {
        let stmt = Query::select()
            .column(Alias::new("from_node"))
            .column(Alias::new("to_node"))
            .from(Alias::new(&format_edge_table_name(of)))
            .to_owned();

        let builder = db.get_database_backend();

        Self::find_by_statement(builder.build(&stmt)).all(db).await
    }
}
