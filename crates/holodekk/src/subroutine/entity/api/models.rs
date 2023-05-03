use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::subroutine::{entity::Status, Entity};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NewSubroutine {
    pub subroutine_image_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Subroutine {
    pub id: String,
    pub scene_entity_id: String,
    pub subroutine_image_id: String,
    pub status: Status,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<Entity> for Subroutine {
    fn from(entity: Entity) -> Self {
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
