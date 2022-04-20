//! Define relation schema

use super::{format_node_table_name, Schema};
use crate::{
    entities::relation,
    lang::{
        iden::{EdgeIden, NodeIden},
        RelationJson,
    },
};
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DbConn, DbErr, DeriveIden, ForeignKeyAction, Set,
};
use sea_query::{Alias, ColumnDef, ForeignKey, Index, Table};

impl Schema {
    /// Insert metadata of relation into database and create a corresponding node table
    pub async fn create_relation(db: &DbConn, relation_json: RelationJson) -> Result<(), DbErr> {
        relation::ActiveModel {
            name: Set(relation_json.name.clone()),
            from_entity: Set(relation_json.from_entity.clone()),
            to_entity: Set(relation_json.to_entity.clone()),
            directed: Set(relation_json.directed),
            ..Default::default()
        }
        .insert(db)
        .await?;

        let edge_name = relation_json.name.as_str();
        let from_entity = format_node_table_name(&relation_json.from_entity);
        let to_entity = format_node_table_name(&relation_json.to_entity);

        Self::add_in_connectivity_columns(db, edge_name, to_entity.as_str()).await?;
        Self::add_out_connectivity_columns(db, edge_name, from_entity.as_str()).await?;

        Self::create_edge_table(db, &relation_json, from_entity, to_entity).await
    }

    async fn add_in_connectivity_columns(
        db: &DbConn,
        edge_name: &str,
        node_table: &str,
    ) -> Result<(), DbErr> {
        for col in [
            "in_conn",
            "in_conn_compound",
            "in_conn_complex03",
            "in_conn_complex05",
            "in_conn_complex07",
        ] {
            Self::add_connectivity_column(db, node_table, &format!("{}_{}", edge_name, col))
                .await?;
        }

        Ok(())
    }

    async fn add_out_connectivity_columns(
        db: &DbConn,
        edge_name: &str,
        node_table: &str,
    ) -> Result<(), DbErr> {
        Self::add_connectivity_column(db, node_table, &format!("{}_out_conn", edge_name)).await
    }

    async fn add_connectivity_column(
        db: &DbConn,
        node_table: &str,
        col: &str,
    ) -> Result<(), DbErr> {
        let builder = db.get_database_backend();
        let mut stmt = Table::alter();
        stmt.table(Alias::new(node_table)).add_column(
            ColumnDef::new(Alias::new(col))
                .double()
                .not_null()
                .default(0.0f64),
        );
        db.execute(builder.build(&stmt)).await?;

        let mut stmt = Index::create();
        stmt.name(&format!("idx-{}-{}", node_table, col))
            .table(Alias::new(node_table))
            .col(Alias::new(col));
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }

    async fn create_edge_table(
        db: &DbConn,
        relation_json: &RelationJson,
        from_entity: String,
        to_entity: String,
    ) -> Result<(), DbErr> {
        let table = Alias::new(relation_json.get_table_name().as_str());
        let mut stmt = Table::create();
        stmt.table(table.clone())
            .col(
                ColumnDef::new(EdgeIden::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(EdgeIden::FromNode).string().not_null())
            .col(ColumnDef::new(EdgeIden::ToNode).string().not_null())
            .index(
                Index::create()
                    .name(&format!("idx-{}-{}", table.to_string(), "from_node"))
                    .table(table.clone())
                    .col(EdgeIden::FromNode),
            )
            .index(
                Index::create()
                    .name(&format!("idx-{}-{}", table.to_string(), "to_node"))
                    .table(table.clone())
                    .col(EdgeIden::ToNode),
            )
            .index(
                Index::create()
                    .unique()
                    .name(&format!(
                        "idx-{}-from_node-to_node",
                        relation_json.get_table_name()
                    ))
                    .col(EdgeIden::FromNode)
                    .col(EdgeIden::ToNode),
            )
            .foreign_key(
                ForeignKey::create()
                    .name(&format!(
                        "fk-{}-from-{}",
                        relation_json.get_table_name(),
                        from_entity
                    ))
                    .from_tbl(Alias::new(relation_json.get_table_name().as_str()))
                    .from_col(EdgeIden::FromNode)
                    .to_tbl(Alias::new(from_entity.as_str()))
                    .to_col(NodeIden::Name)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .foreign_key(
                ForeignKey::create()
                    .name(&format!(
                        "fk-{}-to-{}",
                        relation_json.get_table_name(),
                        to_entity
                    ))
                    .from_tbl(Alias::new(relation_json.get_table_name().as_str()))
                    .from_col(EdgeIden::ToNode)
                    .to_tbl(Alias::new(to_entity.as_str()))
                    .to_col(NodeIden::Name)
                    .on_delete(ForeignKeyAction::Cascade),
            );

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }
}
