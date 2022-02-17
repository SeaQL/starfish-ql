mod common;

use std::collections::{HashMap, HashSet};

use common::TestContext;
use migration::sea_orm::{ConnectionTrait, DbConn, FromQueryResult};
use migration::{Migrator, MigratorTrait, SchemaManager};
use sea_orm::DbErr;
use starfish_core::lang::mutate::{
    MutateDeleteJson, MutateEdgeContentJson, MutateEdgeSelectorJson, MutateInsertJson, MutateJson,
    MutateNodeSelectorJson, MutateUpdateJson,
};
use starfish_core::lang::query::{
    QueryCommonConstraint, QueryConstraintSortByJson, QueryConstraintSortByKeyJson,
    QueryConstraintTraversalJson, QueryGraphConstraint, QueryGraphConstraintJson,
    QueryGraphConstraintLimitJson, QueryGraphJson, QueryJson, QueryResultJson,
    QueryVectorConstraintJson, QueryVectorJson,
};
use starfish_core::lang::schema::{SchemaDefineJson, SchemaJson};
use starfish_core::lang::{ConnectivityTypeJson, Edge, EdgeJsonBatch, Node, NodeJsonBatch};
use starfish_core::mutate::Mutate;
use starfish_core::query::{Query, QueryResultEdge};
use starfish_core::schema::{format_edge_table_name, format_node_table_name};
use starfish_core::sea_query::{Alias, Cond, Expr};
use starfish_core::{
    entities::entity_attribute::Datatype,
    lang::{EntityAttrJson, EntityJson, RelationJson},
    schema::Schema,
};
use starfish_core::{sea_orm, sea_query};

#[allow(unused)]
fn f64_approximately<FA, FB>(a: FA, b: FB) -> bool
where
    FA: Into<f64>,
    FB: Into<f64>,
{
    (a.into() - b.into()).abs() <= f64::EPSILON
}

/// For testing the 'schema' and 'mutate' endpoints
#[smol_potat::test]
async fn schema_mutate() -> Result<(), DbErr> {
    let ctx = TestContext::new("starfish_core_schema_mutate").await;
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
        let stmt = sea_query::Query::select()
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
        let stmt = sea_query::Query::select()
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
        let stmt = sea_query::Query::select()
            .column(Alias::new("from_node"))
            .column(Alias::new("to_node"))
            .from(Alias::new(&format_edge_table_name(of)))
            .to_owned();

        let builder = db.get_database_backend();

        Self::find_by_statement(builder.build(&stmt)).all(db).await
    }
}

/// Specifically for testing the 'query' endpoint.
/// Assumes that the 'schema' and 'mutate' endpoints work correctly.
#[smol_potat::test]
async fn query() -> Result<(), DbErr> {
    let ctx = TestContext::new("starfish_core_query").await;
    let db = &ctx.db;

    Migrator::fresh(db).await?;

    let schema_json = SchemaJson {
        define: SchemaDefineJson {
            entities: vec![EntityJson {
                name: "letter".to_owned(),
                attributes: vec![],
            }],
            relations: vec![RelationJson {
                name: "likes".to_owned(),
                from_entity: "letter".to_owned(),
                to_entity: "letter".to_owned(),
                directed: true,
            }],
        },
    };

    Schema::define_schema(db, schema_json).await?;

    construct_mock_graph(db).await?;

    query_vector(db).await?;

    query_graph_normal(db).await?;

    query_graph_reversed(db).await?;

    query_graph_limited_batch_size(db).await?;

    Ok(())
}

async fn construct_mock_graph(db: &DbConn) -> Result<(), DbErr> {
    let insert_nodes_json = MutateJson::insert(MutateInsertJson::node(NodeJsonBatch {
        of: "letter".to_owned(),
        nodes: Node::new_vec(vec!["A", "B", "C", "D", "E", "F"]),
    }));

    Mutate::mutate(db, insert_nodes_json, false).await?;

    let insert_edges_json = MutateJson::insert(MutateInsertJson::edge(EdgeJsonBatch {
        of: "likes".to_owned(),
        edges: Edge::new_vec(vec![
            ("A", "C"),
            ("B", "C"),
            ("C", "E"),
            ("D", "E"),
            ("D", "F"),
            ("E", "F"),
            ("F", "D"),
            ("F", "E"),
        ]),
    }));

    Mutate::mutate(db, insert_edges_json, false).await?;

    Mutate::calculate_simple_connectivity(db, "likes", "letter", "letter").await?;

    Ok(())
}

async fn query_vector(db: &DbConn) -> Result<(), DbErr> {
    let query_json = QueryJson::Vector(QueryVectorJson {
        of: "letter".to_owned(),
        constraints: vec![
            // Sort by simple in_conn
            QueryVectorConstraintJson::Common(QueryCommonConstraint::SortBy(
                QueryConstraintSortByJson {
                    key: QueryConstraintSortByKeyJson::Connectivity {
                        of: "likes".to_owned(),
                        r#type: ConnectivityTypeJson::Simple,
                    },
                    desc: true,
                },
            )),
        ],
    });

    if let QueryResultJson::Vector(nodes) = Query::query(db, query_json).await? {
        assert_eq!(nodes.len(), 6);

        // Assert that the most liked letter is 'E'.
        assert_eq!(nodes[0].name, "E");
        assert!(f64_approximately(nodes[0].weight.unwrap(), 3));

        // Assert that the fetched weights of the remaining letters are correct.
        for node in nodes.into_iter().skip(2) {
            match node.name.as_str() {
                "A" => assert!(f64_approximately(node.weight.unwrap(), 0)),
                "B" => assert!(f64_approximately(node.weight.unwrap(), 0)),
                "C" => assert!(f64_approximately(node.weight.unwrap(), 2)),
                "D" => assert!(f64_approximately(node.weight.unwrap(), 1)),
                "F" => assert!(f64_approximately(node.weight.unwrap(), 2)),
                _ => panic!("An unknown letter is fetched."),
            }
        }
    } else {
        panic!("Query result should be a Vector.");
    }

    println!("Queried vector successfully.");

    Ok(())
}

async fn query_graph_normal(db: &DbConn) -> Result<(), DbErr> {
    let query_json = QueryJson::Graph(QueryGraphJson {
        of: "letter".to_owned(),
        constraints: vec![
            // Use the edges in "likes"
            QueryGraphConstraintJson::Exclusive(QueryGraphConstraint::Edge {
                of: "likes".to_owned(),
                traversal: QueryConstraintTraversalJson {
                    reverse_direction: false,
                },
            }),
            // Use "A" as root node
            QueryGraphConstraintJson::Exclusive(QueryGraphConstraint::RootNodes(vec![
                "A".to_owned()
            ])),
            // Set max depth as 2
            QueryGraphConstraintJson::Exclusive(QueryGraphConstraint::Limit(
                QueryGraphConstraintLimitJson::Depth(Some(2)),
            )),
        ],
    });

    if let QueryResultJson::Graph { nodes, edges } = Query::query(db, query_json).await? {
        assert_eq!(nodes.len(), 3);
        assert_eq!(edges.len(), 2);

        let nodes: HashSet<String> = HashSet::from_iter(nodes.into_iter().map(|node| node.name));
        let edges: HashSet<QueryResultEdge> = HashSet::from_iter(edges.into_iter());

        // Assert the uniqueness of elements in nodes and edges
        assert_eq!(nodes.len(), 3);
        assert_eq!(edges.len(), 2);

        // Assert that the fetched nodes in the graph are correct
        ["A", "C", "E"].into_iter().for_each(|node| {
            assert!(nodes.contains(node));
        });

        // Assert that the fetched edges in the graph are correct
        [
            QueryResultEdge {
                from_node: "A".to_owned(),
                to_node: "C".to_owned(),
            },
            QueryResultEdge {
                from_node: "C".to_owned(),
                to_node: "E".to_owned(),
            },
        ]
        .into_iter()
        .for_each(|edge| {
            assert!(edges.contains(&edge));
        });
    } else {
        panic!("Query result should be a Graph.");
    }

    println!("Queried normal graph successfully.");

    Ok(())
}

async fn query_graph_reversed(db: &DbConn) -> Result<(), DbErr> {
    let query_json = QueryJson::Graph(QueryGraphJson {
        of: "letter".to_owned(),
        constraints: vec![
            // Use the edges in "likes", but reversed
            QueryGraphConstraintJson::Exclusive(QueryGraphConstraint::Edge {
                of: "likes".to_owned(),
                traversal: QueryConstraintTraversalJson {
                    reverse_direction: true,
                },
            }),
            // Use "E" as root node
            QueryGraphConstraintJson::Exclusive(QueryGraphConstraint::RootNodes(vec![
                "E".to_owned()
            ])),
            // Set max depth as None (no limit)
            // An incorrect implementation leads to infinite loop in cycle traversal
            QueryGraphConstraintJson::Exclusive(QueryGraphConstraint::Limit(
                QueryGraphConstraintLimitJson::Depth(None),
            )),
        ],
    });

    if let QueryResultJson::Graph { nodes, edges } = Query::query(db, query_json).await? {
        assert_eq!(nodes.len(), 6);
        assert_eq!(edges.len(), 8);

        let nodes: HashSet<String> = HashSet::from_iter(nodes.into_iter().map(|node| node.name));
        let edges: HashSet<QueryResultEdge> = HashSet::from_iter(edges.into_iter());

        // Assert the uniqueness of elements in nodes and edges
        assert_eq!(nodes.len(), 6);
        assert_eq!(edges.len(), 8);

        // Assert that the fetched nodes in the graph are correct
        ["A", "B", "C", "D", "E", "F"].into_iter().for_each(|node| {
            assert!(nodes.contains(node));
        });

        // Assert that the fetched edges in the graph are correct
        [
            ("F", "D"),
            ("F", "E"),
            ("E", "C"),
            ("E", "D"),
            ("E", "F"),
            ("D", "F"),
            ("C", "A"),
            ("C", "B"),
        ]
        .into_iter()
        .map(|(from, to)| QueryResultEdge {
            from_node: from.to_owned(),
            to_node: to.to_owned(),
        })
        .for_each(|edge| {
            assert!(edges.contains(&edge));
        });
    } else {
        panic!("Query result should be a Graph.");
    }

    println!("Queried reversed graph successfully.");

    Ok(())
}

async fn query_graph_limited_batch_size(db: &DbConn) -> Result<(), DbErr> {
    let query_json = QueryJson::Graph(QueryGraphJson {
        of: "letter".to_owned(),
        constraints: vec![
            // Use the edges in "likes", but reversed
            QueryGraphConstraintJson::Exclusive(QueryGraphConstraint::Edge {
                of: "likes".to_owned(),
                traversal: QueryConstraintTraversalJson {
                    reverse_direction: true,
                },
            }),
            // Use "E" as root node
            QueryGraphConstraintJson::Exclusive(QueryGraphConstraint::RootNodes(vec![
                "E".to_owned()
            ])),
            // Set max depth as 1
            QueryGraphConstraintJson::Exclusive(QueryGraphConstraint::Limit(
                QueryGraphConstraintLimitJson::Depth(Some(1)),
            )),
            // Set max batch size as 2
            QueryGraphConstraintJson::Exclusive(QueryGraphConstraint::Limit(
                QueryGraphConstraintLimitJson::BatchSize(Some(2)),
            )),
            // Sort by simple in_conn descendingly
            QueryGraphConstraintJson::Common(QueryCommonConstraint::SortBy(
                QueryConstraintSortByJson {
                    key: QueryConstraintSortByKeyJson::Connectivity {
                        of: "likes".to_owned(),
                        r#type: ConnectivityTypeJson::Simple,
                    },
                    desc: true,
                },
            )),
        ],
    });

    if let QueryResultJson::Graph { nodes, edges } = Query::query(db, query_json).await? {
        assert_eq!(nodes.len(), 3);
        assert_eq!(edges.len(), 2);

        let nodes: HashSet<String> = HashSet::from_iter(nodes.into_iter().map(|node| node.name));
        let edges: HashSet<QueryResultEdge> = HashSet::from_iter(edges.into_iter());

        // Assert the uniqueness of elements in nodes and edges
        assert_eq!(nodes.len(), 3);
        assert_eq!(edges.len(), 2);

        // Assert that the fetched nodes in the graph are correct
        ["C", "E", "F"].into_iter().for_each(|node| {
            assert!(nodes.contains(node));
        });

        // Assert that the fetched edges in the graph are correct
        [
            ("E", "C"),
            ("E", "F"),
        ]
        .into_iter()
        .map(|(from, to)| QueryResultEdge {
            from_node: from.to_owned(),
            to_node: to.to_owned(),
        })
        .for_each(|edge| {
            assert!(edges.contains(&edge));
        });
    } else {
        panic!("Query result should be a Graph.");
    }

    println!("Queried limited batch size graph successfully.");

    Ok(())
}
