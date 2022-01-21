mod common;

use common::TestContext;
use sea_orm::DbErr;
use starfish::{api::db::schema::create_tables, schema::Schema, core::{lang::{SchemaJson, EntityJson, EntityAttrJson, RelationJson, SchemaDefineJson}, entities::entity_attribute::Datatype}};

#[smol_potat::test]
async fn schema() -> Result<(), DbErr> {
    let ctx = TestContext::new("schema").await;
    let db = &ctx.db;
    create_tables(db).await?;

    let schema_json = SchemaJson {
        define: SchemaDefineJson {
            entities: vec![
                EntityJson {
                    name: "crate".to_owned(),
                    attributes: vec![EntityAttrJson {
                        name: "version".to_owned(),
                        datatype: Datatype::String,
                    }],
                }
            ],
            relations: vec![
                RelationJson {
                    name: "depends".to_owned(),
                    from_entity: "crate".to_owned(),
                    to_entity: "crate".to_owned(),
                    directed: true,
                }
            ]
        }
    };

    Schema::define_schema(db, schema_json).await?;

    Ok(())
}