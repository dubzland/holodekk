mod name;
pub use name::*;
mod repository;
pub use repository::*;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use timestamps::Timestamps;

use crate::core::enums::SceneStatus;

use super::EntityId;

pub type SceneEntityId = EntityId;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Timestamps)]
pub struct SceneEntity {
    pub id: SceneEntityId,
    pub name: SceneName,
    pub status: SceneStatus,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl Default for SceneEntity {
    fn default() -> Self {
        Self {
            id: SceneEntityId::generate(),
            name: "".into(),
            status: SceneStatus::Unknown,
            created_at: None,
            updated_at: None,
        }
    }
}

impl SceneEntity {
    pub fn new(name: SceneName) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}
