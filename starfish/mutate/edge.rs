use super::Mutate;
use crate::schema::format_edge_table_name;
use sea_orm::{ConnectionTrait, DbConn, DbErr, Statement};
use sea_query::{Alias, Expr, Query};
use serde::{Deserialize, Serialize};

/// Metadata of a edge, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EdgeJson {
    /// Name of relation
    pub name: String,
    /// Name of related node (from side)
    pub from_node: String,
    /// Name of related node (to side)
    pub to_node: String,
}

/// Metadata of edges in batch, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EdgeJsonBatch {
    /// Name of relation
    pub name: String,
    /// Vector of edges
    pub edges: Vec<Edge>,
}

/// Metadata of a edge in batch, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Edge {
    /// Name of related node (from side)
    pub from_node: String,
    /// Name of related node (to side)
    pub to_node: String,
}

/// Metadata of a edge, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClearEdgeJson {
    /// Name of relation
    pub name: String,
    /// Name of node
    pub node: String,
}

impl Mutate {
    /// Insert edge
    pub async fn insert_edge(db: &DbConn, edge_json: EdgeJson) -> Result<(), DbErr> {
        Self::insert_edge_batch(
            db,
            EdgeJsonBatch {
                name: edge_json.name,
                edges: vec![Edge {
                    from_node: edge_json.from_node,
                    to_node: edge_json.to_node,
                }],
            },
        )
        .await
    }

    /// Insert edge in batch
    pub async fn insert_edge_batch(
        db: &DbConn,
        edge_json_batch: EdgeJsonBatch,
    ) -> Result<(), DbErr> {
        let mut stmt = Query::insert();
        stmt.into_table(Alias::new(&format_edge_table_name(edge_json_batch.name)))
            .columns([Alias::new("from_node"), Alias::new("to_node")]);

        for edge_json in edge_json_batch.edges.into_iter() {
            stmt.values_panic([edge_json.from_node.into(), edge_json.to_node.into()]);
        }

        let builder = db.get_database_backend();
        let mut stmt = builder.build(&stmt);
        stmt.sql = stmt.sql.replace("INSERT", "INSERT IGNORE");
        db.execute(stmt).await?;

        Self::update_connectivity(db).await?;

        Ok(())
    }

    /// Delete edge
    pub async fn delete_edge(db: &DbConn, edge_json: EdgeJson) -> Result<(), DbErr> {
        let mut stmt = Query::delete();
        stmt.from_table(Alias::new(&format_edge_table_name(edge_json.name)))
            .and_where(Expr::col(Alias::new("from_node")).eq(edge_json.from_node))
            .and_where(Expr::col(Alias::new("to_node")).eq(edge_json.to_node));

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Self::update_connectivity(db).await?;

        Ok(())
    }

    /// Clear edge
    pub async fn clear_edge(db: &DbConn, clear_edge_json: ClearEdgeJson) -> Result<(), DbErr> {
        let mut stmt = Query::delete();
        stmt.from_table(Alias::new(&format_edge_table_name(clear_edge_json.name)))
            .and_where(Expr::col(Alias::new("from_node")).eq(clear_edge_json.node));

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Self::update_connectivity(db).await?;

        Ok(())
    }

    async fn update_connectivity(db: &DbConn) -> Result<(), DbErr> {
        db.execute(Statement::from_string(db.get_database_backend(), [
            "UPDATE node_crate",
            "SET in_conn = (SELECT COUNT(*) FROM edge_depends WHERE to_node = node_crate.name),",
            "out_conn = (SELECT COUNT(*) FROM edge_depends WHERE from_node = node_crate.name)",
        ].join(" "))).await?;

        Ok(())
    }
}
