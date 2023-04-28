use serde::{Deserialize, Serialize};

use crate::core::entities::SceneId;
use crate::core::enums::ProjectorStatus;
use crate::repositories::EntityId;

pub type ProjectorId = EntityId;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ProjectorEntity {
    pub id: ProjectorId,
    pub scene_id: SceneId,
    pub status: ProjectorStatus,
}

impl Default for ProjectorEntity {
    fn default() -> Self {
        Self {
            id: ProjectorId::generate(),
            scene_id: "".into(),
            status: ProjectorStatus::Unknown,
        }
    }
}

impl ProjectorEntity {
    pub fn new(scene_id: &SceneId, status: &ProjectorStatus) -> Self {
        Self {
            id: EntityId::generate(),
            // scene_id: scene_id.into(),
            scene_id,
            status: status.to_owned(),
        }
    }
}
