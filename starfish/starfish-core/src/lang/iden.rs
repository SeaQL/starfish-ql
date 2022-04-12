use sea_query::Iden;

/// Reusable column identifiers for edges
#[derive(Copy, Clone, Debug, Iden)]
pub enum EdgeIden {
    /// The `from_node` column of an edge
    #[iden = "from_node"]
    FromNode,
    /// The `to_node` column of an edge
    #[iden = "to_node"]
    ToNode,
}

/// Reusable column identifiers for nodes
#[derive(Debug, Iden)]
pub enum NodeIden {
    /// The `name` column of a node
    #[iden = "name"]
    Name,
}

/// Reusable column identifiers for node query results
#[derive(Debug, Iden)]
pub enum NodeQueryIden {
    /// The `name` column of a node query result
    #[iden = "name"]
    Name,
    /// The `weight` column of a node query result
    #[iden = "weight"]
    Weight,
    /// The `depth` column of a node query result
    #[iden = "depth"]
    Depth,
}

/// Reusable column identifiers for entities
#[derive(Debug, Iden)]
pub enum EntityIden {
    /// The `entity` table
    #[iden = "entity"]
    Entity,
    /// The `id` column of an entity
    #[iden = "id"]
    Id,
    /// The `datatype` column of an entity
    #[iden = "datatype"]
    Datatype,
    /// The `name` column of an entity
    #[iden = "name"]
    Name,
}

impl EntityIden {
    /// Specify `self` as a column of the `entity` table
    pub fn prefixed_with_table(self) -> (Self, Self) {
        (Self::Entity, self)
    }
}

/// Reusable column identifiers for entity attributes
#[derive(Debug, Iden)]
pub enum EntityAttrIden {
    /// The `entity_attribute` table
    #[iden = "entity_attribute"]
    EntityAttribute,
    /// The `name` column of an entity attribute
    #[iden = "name"]
    Name,
    /// The `entity_id` column of an entity attribute
    #[iden = "entity_id"]
    EntityId,
}

impl EntityAttrIden {
    /// Specify `self` as a column of the `entity_attribute` table
    pub fn prefixed_with_table(self) -> (Self, Self) {
        (Self::EntityAttribute, self)
    }
}
