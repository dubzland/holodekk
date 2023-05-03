pub mod api;
pub mod repository;
pub use repository::Repository;
pub mod service;
pub use service::Service;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use timestamps::Timestamps;

use crate::images::SubroutineImageId;

use crate::entity;
use crate::scene;

pub type Id = entity::Id;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum Status {
    Unknown,
    Stopped,
    Running(u32),
    Crashed,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Timestamps)]
pub struct Entity {
    pub id: Id,
    pub scene_entity_id: scene::entity::Id,
    pub subroutine_image_id: SubroutineImageId,
    pub status: Status,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl Entity {
    pub fn new(
        scene_entity_id: &scene::entity::Id,
        subroutine_image_id: &SubroutineImageId,
    ) -> Self {
        Self {
            id: Id::generate(),
            scene_entity_id: scene_entity_id.clone(),
            subroutine_image_id: subroutine_image_id.clone(),
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

    use crate::images::{fixtures::mock_subroutine_image, SubroutineImage};
    use crate::scene::{entity::mock_entity as mock_scene_entity, Entity as SceneEntity};

    use super::*;

    #[fixture]
    pub fn mock_entity(
        mock_scene_entity: SceneEntity,
        mock_subroutine_image: SubroutineImage,
    ) -> Entity {
        let mut subroutine = Entity::new(&mock_scene_entity.id, &mock_subroutine_image.id);
        subroutine.created_at = Some(Utc::now().naive_utc());
        subroutine
    }
}

#[cfg(test)]
pub use fixtures::*;
