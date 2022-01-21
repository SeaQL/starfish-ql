mod common;

use common::TestContext;
use migration::{Migrator, MigratorTrait, SchemaManager};
use sea_orm::DbErr;
use starfish_core::sea_orm;
use starfish_core::{
    entities::entity_attribute::Datatype,
    lang::{EntityAttrJson, EntityJson, RelationJson, SchemaDefineJson, SchemaJson},
    schema::Schema,
};

#[smol_potat::test]
async fn schema() -> Result<(), DbErr> {
    let ctx = TestContext::new("schema").await;
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

    Ok(())
}
