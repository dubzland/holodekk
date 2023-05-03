pub mod api;
pub mod repository;
pub use repository::Repository;
pub mod service;
pub use service::Service;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use timestamps::Timestamps;

use crate::entity;
use crate::scene;
use crate::subroutine;

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
    pub image_id: subroutine::image::Id,
    pub status: Status,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl Entity {
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

    use crate::scene::{entity::mock_entity as mock_scene_entity, Entity as SceneEntity};
    use crate::subroutine::image::{fixtures::mock_image, Image};

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
