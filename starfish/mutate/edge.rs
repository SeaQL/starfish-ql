use super::{Mutate, NodeJson};
use crate::{
    core::entities::{entity, relation},
    schema::{format_edge_table_name, format_node_table_name},
};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbConn, DbErr, EntityTrait, FromQueryResult, QueryFilter,
};
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

/// Metadata of a edge, deserialized as struct from json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClearEdgeJson {
    /// Name of relation
    pub name: String,
    /// Name of node
    pub node: String,
}

/// Storing temporary query result
#[derive(Debug, FromQueryResult)]
struct Node {
    id: i32,
}

impl Mutate {
    /// Insert edge
    pub async fn insert_edge(db: &DbConn, edge_json: EdgeJson) -> Result<(), DbErr> {
        let (relation_name, from_node, to_node) = Self::get_nodes(db, edge_json).await?;

        let mut stmt = Query::insert();
        stmt.into_table(Alias::new(&format_edge_table_name(relation_name)))
            .columns([Alias::new("from_node_id"), Alias::new("to_node_id")])
            .values_panic([from_node.id.into(), to_node.id.into()]);

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }

    /// Delete edge
    pub async fn delete_edge(db: &DbConn, edge_json: EdgeJson) -> Result<(), DbErr> {
        let (relation_name, from_node, to_node) = Self::get_nodes(db, edge_json).await?;

        let mut stmt = Query::delete();
        stmt.from_table(Alias::new(&format_edge_table_name(relation_name)))
            .and_where(Expr::col(Alias::new("from_node_id")).eq(from_node.id))
            .and_where(Expr::col(Alias::new("to_node_id")).eq(to_node.id));

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }

    /// Clear edge
    pub async fn clear_edge(db: &DbConn, clear_edge_json: ClearEdgeJson) -> Result<(), DbErr> {
        let builder = db.get_database_backend();

        let relation = relation::Entity::find()
            .filter(relation::Column::Name.eq(clear_edge_json.name.as_str()))
            .one(db)
            .await?
            .ok_or_else(|| {
                DbErr::Custom(format!(
                    "Relation of name '{}' could not be found",
                    clear_edge_json.name
                ))
            })?;

        let from_entity = entity::Entity::find_by_id(relation.from_entity_id)
            .one(db)
            .await?
            .ok_or_else(|| {
                DbErr::Custom(format!(
                    "Entity of id '{}' could not be found",
                    relation.from_entity_id
                ))
            })?;

        let mut stmt = Query::select();
        stmt.expr(Expr::col(Alias::new("id")))
            .from(Alias::new(&format_node_table_name(
                from_entity.name.as_str(),
            )))
            .and_where(Expr::col(Alias::new("name")).eq(clear_edge_json.node.as_str()));
        let from_node = Node::find_by_statement(builder.build(&stmt))
            .one(db)
            .await?
            .ok_or_else(|| {
                DbErr::Custom(format!(
                    "Node of name '{}' could not be found",
                    clear_edge_json.node
                ))
            })?;

        let mut stmt = Query::delete();
        stmt.from_table(Alias::new(&format_edge_table_name(clear_edge_json.name)))
            .and_where(Expr::col(Alias::new("from_node_id")).eq(from_node.id));

        let builder = db.get_database_backend();
        db.execute(builder.build(&stmt)).await?;

        Ok(())
    }

    async fn get_nodes(db: &DbConn, edge_json: EdgeJson) -> Result<(String, Node, Node), DbErr> {
        let relation = relation::Entity::find()
            .filter(relation::Column::Name.eq(edge_json.name.as_str()))
            .one(db)
            .await?
            .ok_or_else(|| {
                DbErr::Custom(format!(
                    "Relation of name '{}' could not be found",
                    edge_json.name
                ))
            })?;

        let from_entity = entity::Entity::find_by_id(relation.from_entity_id)
            .one(db)
            .await?
            .ok_or_else(|| {
                DbErr::Custom(format!(
                    "Entity of id '{}' could not be found",
                    relation.from_entity_id
                ))
            })?;

        let to_entity = entity::Entity::find_by_id(relation.to_entity_id)
            .one(db)
            .await?
            .ok_or_else(|| {
                DbErr::Custom(format!(
                    "Entity of id '{}' could not be found",
                    relation.to_entity_id
                ))
            })?;

        let builder = db.get_database_backend();

        let mut stmt = Query::select();
        stmt.expr(Expr::col(Alias::new("id")))
            .from(Alias::new(&format_node_table_name(
                from_entity.name.as_str(),
            )))
            .and_where(Expr::col(Alias::new("name")).eq(edge_json.from_node.as_str()));
        let from_node = Node::find_by_statement(builder.build(&stmt))
            .one(db)
            .await?
            .ok_or_else(|| {
                DbErr::Custom(format!(
                    "Node of name '{}' could not be found",
                    edge_json.from_node
                ))
            })?;

        let to_node =
            if let Some(to_node) = Self::try_get_to_node(db, &to_entity, &edge_json).await? {
                to_node
            } else {
                Mutate::insert_node(
                    db,
                    NodeJson {
                        of: to_entity.name.to_owned(),
                        name: edge_json.to_node.to_owned(),
                        attributes: Default::default(),
                    },
                )
                .await?;
                Self::try_get_to_node(db, &to_entity, &edge_json)
                    .await?
                    .ok_or_else(|| {
                        DbErr::Custom(format!(
                            "Node of name '{}' could not be found",
                            edge_json.to_node
                        ))
                    })?
            };

        Ok((relation.name, from_node, to_node))
    }

    async fn try_get_to_node(
        db: &DbConn,
        to_entity: &entity::Model,
        edge_json: &EdgeJson,
    ) -> Result<Option<Node>, DbErr> {
        let builder = db.get_database_backend();

        let mut stmt = Query::select();
        stmt.expr(Expr::col(Alias::new("id")))
            .from(Alias::new(&format_node_table_name(to_entity.name.as_str())))
            .and_where(Expr::col(Alias::new("name")).eq(edge_json.to_node.as_str()));
        let to_node = Node::find_by_statement(builder.build(&stmt))
            .one(db)
            .await?;

        Ok(to_node)
    }
}
