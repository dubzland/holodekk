use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::entities::SceneEntity;
use crate::enums::SceneStatus;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NewScene {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Scene {
    pub id: String,
    pub name: String,
    pub status: SceneStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<SceneEntity> for Scene {
    fn from(entity: SceneEntity) -> Self {
        Self {
            id: entity.id.into(),
            name: entity.name.into(),
            status: entity.status,
            created_at: entity.created_at.unwrap(),
            updated_at: entity.updated_at,
        }
    }
}
