//! DTO's for the `subroutine` api

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::scene::{entity::Status, Entity};

/// Data provided to construct a new scene [`Entity`]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NewScene {
    /// user-assigned name for this scene
    pub name: String,
}

/// Representation of a `scene` [`Entity`] returned to clients
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Scene {
    /// the entity `Id`
    pub id: String,
    /// the user-assigned name
    pub name: String,
    /// current status of this `scene`
    pub status: Status,
    /// entity creation timestamp from repository
    pub created_at: NaiveDateTime,
    /// optional update timestamp from repository
    pub updated_at: Option<NaiveDateTime>,
}

impl From<Entity> for Scene {
    fn from(entity: Entity) -> Self {
        Self {
            id: entity.id.into(),
            name: entity.name.into(),
            status: entity.status,
            created_at: entity.created_at.unwrap(),
            updated_at: entity.updated_at,
        }
    }
}
