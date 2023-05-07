//! Repository storage representation for a running subroutine instance.

pub mod api;
pub mod repository;
pub use repository::Repository;
pub mod service;
pub use service::Service;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use timestamps::Timestamps;

use crate::core::scene;
use crate::core::subroutine;
use crate::entity;

/// Subroutine specific entity id (data storage key)
pub type Id = entity::Id;

/// Current status of a given subroutine
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum Status {
    /// Unknown
    Unknown,
    /// Stopped
    Stopped,
    /// Currently active (with the specified pid)
    Running(u32),
    /// Stopped without being requested
    Crashed,
}

/// Data storage entity for a given subroutine
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Timestamps)]
pub struct Entity {
    /// Subroutine key
    pub id: Id,
    /// Scene this subroutine instance belongs to
    pub scene_entity_id: scene::entity::Id,
    /// Image this running subroutine is based on
    pub image_id: subroutine::image::Id,
    /// Current status of this subroutine
    pub status: Status,
    /// Timestamp for the subroutine's repository record
    pub created_at: Option<NaiveDateTime>,
    /// Last time this subroutine's repository record was updated
    pub updated_at: Option<NaiveDateTime>,
}

impl Entity {
    /// Constructs a new subroutine entity
    #[must_use]
    pub fn new(scene_entity_id: &scene::entity::Id, image_id: &subroutine::image::Id) -> Self {
        Self {
            id: Id::generate(),
            scene_entity_id: scene_entity_id.clone(),
            image_id: image_id.clone(),
            status: Status::Unknown,
            created_at: None,
            updated_at: None,
        }
    }
}

#[cfg(test)]
mod fixtures {
    use chrono::Utc;
    use rstest::fixture;

    use crate::core::scene::{entity::mock_entity as mock_scene_entity, Entity as SceneEntity};
    use crate::core::subroutine::image::{fixtures::mock_image, Image};

    use super::*;

    #[fixture]
    pub fn mock_entity(mock_scene_entity: SceneEntity, mock_image: Image) -> Entity {
        let mut subroutine = Entity::new(&mock_scene_entity.id, &mock_image.id);
        subroutine.created_at = Some(Utc::now().naive_utc());
        subroutine
    }
}

#[cfg(test)]
pub use fixtures::*;
