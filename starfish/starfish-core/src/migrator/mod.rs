pub use sea_schema::migration::*;

mod m20220121_000001_create_entity_table;
mod m20220121_000002_create_relation_table;
mod m20220121_000003_create_entity_attribute_table;

#[derive(Debug)]
/// To perform database migration
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220121_000001_create_entity_table::Migration),
            Box::new(m20220121_000002_create_relation_table::Migration),
            Box::new(m20220121_000003_create_entity_attribute_table::Migration),
        ]
    }
}
