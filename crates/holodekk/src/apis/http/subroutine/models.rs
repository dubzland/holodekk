use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::entities::SubroutineEntity;
use crate::enums::SubroutineStatus;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NewSubroutine {
    pub subroutine_image_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Subroutine {
    pub id: String,
    pub scene_entity_id: String,
    pub subroutine_image_id: String,
    pub status: SubroutineStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<SubroutineEntity> for Subroutine {
    fn from(entity: SubroutineEntity) -> Self {
        Self {
            id: entity.id.into(),
            scene_entity_id: entity.scene_entity_id.into(),
            subroutine_image_id: entity.subroutine_image_id.into(),
            status: entity.status,
            created_at: entity.created_at.unwrap(),
            updated_at: entity.updated_at,
        }
    }
}
