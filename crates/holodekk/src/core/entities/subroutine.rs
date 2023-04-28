use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use timestamps::Timestamps;

use crate::core::enums::SubroutineStatus;

use super::{EntityId, SceneEntityId, SubroutineDefinitionEntityId};

pub type SubroutineEntityId = EntityId;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Timestamps)]
pub struct SubroutineEntity {
    pub id: SubroutineEntityId,
    pub scene_id: SceneEntityId,
    pub subroutine_definition_id: SubroutineDefinitionEntityId,
    pub status: SubroutineStatus,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl SubroutineEntity {
    pub fn new(
        scene_id: &SceneEntityId,
        subroutine_definition_id: &SubroutineDefinitionEntityId,
    ) -> Self {
        Self {
            id: SubroutineEntityId::generate(),
            scene_id: scene_id.to_owned(),
            subroutine_definition_id: subroutine_definition_id.to_owned(),
            status: SubroutineStatus::Unknown,
            created_at: None,
            updated_at: None,
        }
    }
}
