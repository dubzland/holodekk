use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use serde::{Deserialize, Serialize};

use crate::core::{
    entities::{SceneEntity, SceneEntityId, SceneName},
    enums::SceneStatus,
    repositories::{RepositoryQuery, Result},
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum SceneEvent {
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
pub struct ScenesQuery<'a> {
    name: Option<&'a SceneName>,
    status: Option<&'a SceneStatus>,
}

impl<'a> ScenesQuery<'a> {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn name_eq(&mut self, name: &'a SceneName) -> &mut Self {
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

    pub fn name(&self) -> Option<&'a SceneName> {
        self.name
    }

    pub fn status(&self) -> Option<&SceneStatus> {
        self.status
    }
}

impl<'a> From<&'a SceneEntity> for ScenesQuery<'a> {
    fn from(scene: &'a SceneEntity) -> Self {
        Self::builder()
            .name_eq(&scene.name)
            .status_eq(&scene.status)
            .build()
    }
}

impl<'a> RepositoryQuery for ScenesQuery<'a> {
    type Entity = SceneEntity;

    fn matches(&self, scene: &SceneEntity) -> bool {
        if let Some(name) = self.name {
            &scene.name == name
        } else {
            true
        }
    }
}

impl PartialEq<ScenesQuery<'_>> for SceneEntity {
    fn eq(&self, other: &ScenesQuery) -> bool {
        other.matches(self)
    }
}

impl<'a> PartialEq<SceneEntity> for ScenesQuery<'a> {
    fn eq(&self, other: &SceneEntity) -> bool {
        self.matches(other)
    }
}

// #[derive(Debug)]
// pub struct ScenesWatcher {
//     pub id: WatchId,
//     pub tx: tokio::sync::mpsc::Sender<SceneEvent>,
// }

// impl ScenesWatcher {
//     pub fn new(tx: tokio::sync::mpsc::Sender<SceneEvent>) -> Self {
//         Self {
//             id: WatchId::generate(),
//             tx,
//         }
//     }

//     pub async fn send_insert(&mut self, scene: &SceneEntity) {
//         let event = SceneEvent::Insert {
//             scene: scene.to_owned(),
//         };
//         self.tx.send(event).await.unwrap();
//     }

//     pub async fn send_update(&mut self, scene: &SceneEntity, orig: &SceneEntity) {
//         let event = SceneEvent::Update {
//             scene: scene.to_owned(),
//             orig: orig.to_owned(),
//         };
//         self.tx.send(event).await.unwrap();
//     }

//     pub async fn send_delete(&mut self, value: &SceneEntity) {
//         let event = SceneEvent::Delete {
//             scene: value.to_owned(),
//         };
//         self.tx.send(event).await.unwrap();
//     }
// }

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ScenesRepository: Send + Sync {
    async fn scenes_create(&self, scene: SceneEntity) -> Result<SceneEntity>;
    async fn scenes_delete(&self, id: &SceneEntityId) -> Result<()>;
    async fn scenes_exists<'a>(&self, query: ScenesQuery<'a>) -> Result<bool>;
    async fn scenes_find<'a>(&self, query: ScenesQuery<'a>) -> Result<Vec<SceneEntity>>;
    async fn scenes_get(&self, id: &SceneEntityId) -> Result<SceneEntity>;
    async fn scenes_update(
        &self,
        id: &SceneEntityId,
        name: Option<SceneName>,
        status: Option<SceneStatus>,
    ) -> Result<SceneEntity>;
}
