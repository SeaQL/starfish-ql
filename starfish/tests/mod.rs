mod common;

use common::TestContext;
use sea_orm::{DbConn, DbErr, FromQueryResult, ConnectionTrait};
use sea_query::Alias;
use starfish::{
    api::db::schema::create_tables,
    core::entities::entity_attribute::Datatype,
    mutate::{ClearEdgeJson, EdgeJson, Mutate, NodeJson, NodeJsonBatch, Node, EdgeJsonBatch, Edge},
    schema::{EntityAttrJson, EntityJson, RelationJson, Schema},
};
use std::collections::HashMap;

#[smol_potat::test]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("starfish_tests_main").await;
    let db = &ctx.db;

    create_tables(db).await?;

    test_create_entities(db).await?;
    test_create_relations(db).await?;
    test_insert_node(db).await?;
    test_delete_node(db).await?;
    test_insert_edge(db).await?;
    test_delete_edge(db).await?;
    test_clear_edge(db).await?;

    Ok(())
}

#[smol_potat::test]
async fn connectivity() -> Result<(), DbErr> {
    let ctx = TestContext::new("starfish_tests_connectivity").await;
    let db = &ctx.db;

    create_tables(db).await?;

    test_create_entities(db).await?;
    test_create_relations(db).await?;

    let correct_nodes = test_construct_mock_graph(db).await?;
    Mutate::calculate_compound_connectivity(db).await?;

    let nodes = test_get_nodes_with_connectivity(db).await?;
    assert_eq!(nodes.len(), 6);

    for (name, node) in nodes {
        assert_eq!(node, *correct_nodes.get(&name).unwrap());
    }

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

async fn test_clear_edge(db: &DbConn) -> Result<(), DbErr> {
    Mutate::clear_edge(
        db,
        ClearEdgeJson {
            name: "depends".to_owned(),
            node: "sea-orm".to_owned(),
        },
    )
    .await?;

    Ok(())
}

#[derive(Debug, Clone, Eq, FromQueryResult, PartialEq)]
struct TestNode {
    name: String,
    in_conn: i32,
    in_conn_compound: i32,
}

async fn test_construct_mock_graph(db: &DbConn) -> Result<HashMap<String, TestNode>, DbErr> {
    Mutate::insert_node_batch(
        db,
        NodeJsonBatch {
            of: "crate".to_owned(),
            nodes: vec![
                Node { name: "A".to_owned(), attributes: HashMap::new() },
                Node { name: "B".to_owned(), attributes: HashMap::new() },
                Node { name: "C".to_owned(), attributes: HashMap::new() },
                Node { name: "D".to_owned(), attributes: HashMap::new() },
                Node { name: "E".to_owned(), attributes: HashMap::new() },
                Node { name: "F".to_owned(), attributes: HashMap::new() },
            ],
        }
    )
    .await?;

    Mutate::insert_edge_batch(
        db,
        EdgeJsonBatch {
            name: "depends".to_owned(),
            edges: vec![
                Edge { from_node: "A".to_owned(), to_node: "C".to_owned() },
                Edge { from_node: "B".to_owned(), to_node: "C".to_owned() },
                Edge { from_node: "B".to_owned(), to_node: "D".to_owned() },
                Edge { from_node: "C".to_owned(), to_node: "E".to_owned() },
                Edge { from_node: "D".to_owned(), to_node: "E".to_owned() },
                Edge { from_node: "D".to_owned(), to_node: "F".to_owned() },
            ],
        }
    )
    .await?;

    Ok(
        HashMap::from([
            ("A".to_owned(), TestNode { name: "A".to_owned(), in_conn: 0, in_conn_compound: 0 }),
            ("B".to_owned(), TestNode { name: "B".to_owned(), in_conn: 0, in_conn_compound: 0 }),
            ("C".to_owned(), TestNode { name: "C".to_owned(), in_conn: 2, in_conn_compound: 2 }),
            ("D".to_owned(), TestNode { name: "D".to_owned(), in_conn: 1, in_conn_compound: 1 }),
            ("E".to_owned(), TestNode { name: "E".to_owned(), in_conn: 2, in_conn_compound: 4 }),
            ("F".to_owned(), TestNode { name: "F".to_owned(), in_conn: 1, in_conn_compound: 2 }),
        ])
    )
}

async fn test_get_nodes_with_connectivity(db: &DbConn) -> Result<HashMap<String, TestNode>, DbErr> {
    let builder = db.get_database_backend();
    Ok(
        TestNode::find_by_statement(
            builder.build(
                sea_query::Query::select()
                    .columns([
                        Alias::new("name"),
                        Alias::new("in_conn"),
                        Alias::new("in_conn_compound")
                    ])
                    .from(Alias::new("node_crate"))
            )
        )
        .all(db)
        .await?
        .into_iter()
        .fold(HashMap::new(), |mut map, node| {
            map.insert(node.name.clone(), node);
            map
        })
    )
}