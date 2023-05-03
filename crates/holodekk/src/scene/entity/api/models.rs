use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::scene;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NewScene {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Scene {
    pub id: String,
    pub name: String,
    pub status: scene::entity::Status,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<scene::Entity> for Scene {
    fn from(entity: scene::Entity) -> Self {
        Self {
            id: entity.id.into(),
            name: entity.name.into(),
            status: entity.status,
            created_at: entity.created_at.unwrap(),
            updated_at: entity.updated_at,
        }
    }
}
