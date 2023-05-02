use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use serde::{Deserialize, Serialize};

use crate::core::{
    entities::repository::{EntityRepositoryQuery, EntityRepositoryResult},
    enums::SceneStatus,
};

use super::{SceneEntity, SceneEntityId, SceneName};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum SceneEntityRepositoryEvent {
    Unknown,
    Insert {
        scene: SceneEntity,
    },
    Update {
        scene: SceneEntity,
        orig: SceneEntity,
    },
    Delete {
        scene: SceneEntity,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneEntityRepositoryQuery<'a> {
    name: Option<&'a str>,
    status: Option<&'a SceneStatus>,
}

impl<'a> SceneEntityRepositoryQuery<'a> {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn name_eq(&mut self, name: &'a str) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn status_eq(&mut self, status: &'a SceneStatus) -> &mut Self {
        self.status = Some(status);
        self
    }

    pub fn build(&self) -> Self {
        Self {
            name: self.name,
            status: self.status,
        }
    }

    pub fn name(&self) -> Option<&'a str> {
        self.name
    }

    pub fn status(&self) -> Option<&SceneStatus> {
        self.status
    }
}

impl<'a> From<&'a SceneEntity> for SceneEntityRepositoryQuery<'a> {
    fn from(scene: &'a SceneEntity) -> Self {
        Self::builder()
            .name_eq(&scene.name)
            .status_eq(&scene.status)
            .build()
    }
}

impl<'a> EntityRepositoryQuery for SceneEntityRepositoryQuery<'a> {
    type Entity = SceneEntity;

    fn matches(&self, scene: &SceneEntity) -> bool {
        if let Some(name) = self.name {
            &scene.name == name
        } else {
            true
        }
    }
}

impl PartialEq<SceneEntityRepositoryQuery<'_>> for SceneEntity {
    fn eq(&self, other: &SceneEntityRepositoryQuery) -> bool {
        other.matches(self)
    }
}

impl<'a> PartialEq<SceneEntity> for SceneEntityRepositoryQuery<'a> {
    fn eq(&self, other: &SceneEntity) -> bool {
        self.matches(other)
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait SceneEntityRepository: Send + Sync + 'static {
    async fn scenes_create(&self, scene: SceneEntity) -> EntityRepositoryResult<SceneEntity>;
    async fn scenes_delete(&self, id: &SceneEntityId) -> EntityRepositoryResult<()>;
    async fn scenes_exists<'a>(
        &self,
        query: SceneEntityRepositoryQuery<'a>,
    ) -> EntityRepositoryResult<bool>;
    async fn scenes_find<'a>(
        &self,
        query: SceneEntityRepositoryQuery<'a>,
    ) -> EntityRepositoryResult<Vec<SceneEntity>>;
    async fn scenes_get(&self, id: &SceneEntityId) -> EntityRepositoryResult<SceneEntity>;
    async fn scenes_update(
        &self,
        id: &SceneEntityId,
        name: Option<SceneName>,
        status: Option<SceneStatus>,
    ) -> EntityRepositoryResult<SceneEntity>;
}
