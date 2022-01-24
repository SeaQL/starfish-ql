//! Data manipulation operations

mod edge;
mod node;

pub use edge::*;
pub use node::*;
use sea_orm::{DbConn, DbErr};

use crate::lang::{MutateJson, MutateInsertContentJson, MutateSelectorJson};

/// Mutate node and edge
#[derive(Debug)]
pub struct Mutate;

impl Mutate {
    /// Mutate data in db
    pub async fn mutate(
        db: &DbConn,
        mutate_json: MutateJson,
        upsert: bool
    ) -> Result<(), DbErr> {

        match mutate_json {
            MutateJson::insert(insert_content) => {
                match insert_content {
                    MutateInsertContentJson::node(batch) => {
                        Mutate::insert_node_batch(db, batch, upsert)
                            .await?;
                    },
                    MutateInsertContentJson::edge(batch) => {
                        Mutate::insert_edge_batch(db, batch)
                            .await?;
                    },
                }
            },
            MutateJson::update(selector) => {
                match selector {
                    MutateSelectorJson::node { of, attributes } => todo!(),
                    MutateSelectorJson::edge { of, from_node, to_node } => todo!(),
                }
            },
            MutateJson::delete(selector) => {
                match selector {
                    MutateSelectorJson::node { of, attributes } => todo!(),
                    MutateSelectorJson::edge { of, from_node, to_node } => todo!(),
                }
            },
        };

        Ok(())

    }
}
