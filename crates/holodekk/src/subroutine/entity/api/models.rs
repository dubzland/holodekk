//! DTO's for the `subroutine` api

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::subroutine::{entity::Status, Entity};

/// Data provided to construct a new subroutine [`Entity`]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NewSubroutine {
    /// [`Id`][`crate::subroutine::image::Id`] this subroutine will be running
    pub image_id: String,
}

/// Representation of a `subroutine` [`Entity`] returned to clients
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Subroutine {
    /// the entity `Id`
    pub id: String,
    /// the `Id` of the scene this `subroutine` belongs to
    pub scene_entity_id: String,
    /// the `Id` of the image this `subroutine` is running
    pub image_id: String,
    /// current status of this `subroutine`
    pub status: Status,
    /// entity creation timestamp from repository
    pub created_at: NaiveDateTime,
    /// optional update timestamp from repository
    pub updated_at: Option<NaiveDateTime>,
}

impl From<Entity> for Subroutine {
    fn from(entity: Entity) -> Self {
        Self {
            id: entity.id.into(),
            scene_entity_id: entity.scene_entity_id.into(),
            image_id: entity.image_id.into(),
            status: entity.status,
            created_at: entity.created_at.unwrap(),
            updated_at: entity.updated_at,
        }
    }
}
