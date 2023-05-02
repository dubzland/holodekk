use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use timestamps::Timestamps;

use crate::core::enums::SubroutineStatus;
use crate::core::images::SubroutineImageId;

use super::{EntityId, SceneEntityId};

pub type SubroutineEntityId = EntityId;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Timestamps)]
pub struct SubroutineEntity {
    pub id: SubroutineEntityId,
    pub scene_entity_id: SceneEntityId,
    pub subroutine_image_id: SubroutineImageId,
    pub status: SubroutineStatus,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl SubroutineEntity {
    pub fn new(scene_entity_id: &SceneEntityId, subroutine_image_id: &SubroutineImageId) -> Self {
        Self {
            id: SubroutineEntityId::generate(),
            scene_entity_id: scene_entity_id.to_owned(),
            subroutine_image_id: subroutine_image_id.to_owned(),
            status: SubroutineStatus::Unknown,
            created_at: None,
            updated_at: None,
        }
    }
}
