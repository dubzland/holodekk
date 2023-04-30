use std::ops::Deref;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use timestamps::Timestamps;

use crate::core::actions::scene_create;
use crate::core::enums::SceneStatus;

use super::EntityId;

pub type SceneEntityId = EntityId;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SceneName(String);

impl std::fmt::Display for SceneName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&SceneName> for String {
    fn from(value: &SceneName) -> Self {
        value.0.clone()
    }
}

impl From<&str> for SceneName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for SceneName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl Deref for SceneName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<T> for SceneName
where
    T: ?Sized,
    <SceneName as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

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

impl From<scene_create::Request<'_>> for SceneEntity {
    fn from(req: scene_create::Request) -> Self {
        let mut scene = Self::new(req.name.to_owned());
        scene.status = req.status.to_owned();
        scene
    }
}
