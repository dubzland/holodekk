use async_trait::async_trait;
use etcd_client::GetOptions;
use timestamps::Timestamps;

use crate::core::{
    entities::{
        EntityId, EntityRepositoryError, EntityRepositoryQuery, EntityRepositoryResult,
        SceneEntity, SceneEntityId, SceneEntityRepository, SceneEntityRepositoryEvent,
        SceneEntityRepositoryQuery, SceneName,
    },
    enums::SceneStatus,
};

use super::{etcd_scene_key, EtcdRepository};

impl From<etcd_client::Event> for SceneEntityRepositoryEvent {
    fn from(event: etcd_client::Event) -> Self {
        if let Some(kv) = event.kv() {
            match event.event_type() {
                etcd_client::EventType::Put => {
                    let value = kv.value_str().unwrap();
                    let current: SceneEntity = serde_json::from_str(value).unwrap();
                    if let Some(prev_kv) = event.prev_kv() {
                        let prev_value = prev_kv.value_str().unwrap();
                        let orig: SceneEntity = serde_json::from_str(prev_value).unwrap();

                        Self::Update {
                            scene: current,
                            orig,
                        }
                    } else {
                        Self::Insert { scene: current }
                    }
                }
                etcd_client::EventType::Delete => {
                    let prev_value = event.prev_kv().unwrap().value_str().unwrap();
                    let prev: SceneEntity = serde_json::from_str(prev_value).unwrap();
                    Self::Delete { scene: prev }
                }
            }
        } else {
            Self::Unknown
        }
    }
}

#[async_trait]
impl SceneEntityRepository for EtcdRepository {
    async fn scenes_create(&self, mut scene: SceneEntity) -> EntityRepositoryResult<SceneEntity> {
        match self.scenes_get(&scene.id).await {
            Err(EntityRepositoryError::NotFound(_)) => {
                scene.created();
                scene.updated();
                let serialized = serde_json::to_string(&scene)?;
                let key = etcd_scene_key(Some(&scene.id));
                let mut client = self.client.read().unwrap().clone().unwrap();
                client.put(key, serialized, None).await?;
                Ok(scene)
            }
            Ok(_) => Err(EntityRepositoryError::Conflict(format!(
                "Scene already exists with id {}",
                scene.id
            ))),
            Err(err) => Err(err),
        }
    }

    async fn scenes_delete(&self, id: &EntityId) -> EntityRepositoryResult<()> {
        let mut client = self.client.read().unwrap().clone().unwrap();
        let key = etcd_scene_key(Some(id));
        let result = client
            .get(key.clone(), Some(GetOptions::new().with_count_only()))
            .await?;
        if result.count() == 0 {
            Err(EntityRepositoryError::NotFound(id.to_owned()))
        } else {
            client.delete(key, None).await?;
            Ok(())
        }
    }

    async fn scenes_exists<'a>(
        &self,
        query: SceneEntityRepositoryQuery<'a>,
    ) -> EntityRepositoryResult<bool> {
        let mut client = self.client.read().unwrap().clone().unwrap();
        let key = etcd_scene_key(None);
        let result = client
            .get(key, Some(GetOptions::new().with_prefix()))
            .await?;

        if result.count() == 0 {
            return Ok(false);
        }

        let exists =
            result
                .kvs()
                .iter()
                .any(|v| match serde_json::from_slice::<SceneEntity>(v.value()) {
                    Ok(scene) => query.matches(&scene),
                    Err(_) => false,
                });

        Ok(exists)
    }

    async fn scenes_find<'a>(
        &self,
        query: SceneEntityRepositoryQuery<'a>,
    ) -> EntityRepositoryResult<Vec<SceneEntity>> {
        let mut client = self.client.read().unwrap().clone().unwrap();
        let key = etcd_scene_key(None);
        let result = client
            .get(key, Some(GetOptions::new().with_prefix()))
            .await?;

        let scenes = result
            .kvs()
            .iter()
            .filter_map(|v| match serde_json::from_slice::<SceneEntity>(v.value()) {
                Ok(scene) => {
                    if query.matches(&scene) {
                        Some(scene)
                    } else {
                        None
                    }
                }
                Err(_) => None,
            })
            .collect();

        Ok(scenes)
    }

    async fn scenes_get(&self, id: &EntityId) -> EntityRepositoryResult<SceneEntity> {
        let mut client = self.client.read().unwrap().clone().unwrap();
        let key = etcd_scene_key(Some(id));
        let result = client.get(key, None).await?;

        if result.count() != 1 {
            Err(EntityRepositoryError::NotFound(id.to_owned()))
        } else if let Some(kv) = result.kvs().first() {
            let scene: SceneEntity = serde_json::from_slice(kv.value())?;
            Ok(scene)
        } else {
            Err(EntityRepositoryError::NotFound(id.to_owned()))
        }
    }

    async fn scenes_update(
        &self,
        id: &SceneEntityId,
        name: Option<SceneName>,
        status: Option<SceneStatus>,
    ) -> EntityRepositoryResult<SceneEntity> {
        let mut client = self.client.read().unwrap().clone().unwrap();
        let key = etcd_scene_key(Some(id));
        let result = client.get(key.clone(), None).await?;

        if let Some(kv) = result.kvs().first() {
            let mut scene: SceneEntity = serde_json::from_slice(kv.value())?;
            if let Some(name) = name {
                scene.name = name;
            }
            if let Some(status) = status {
                scene.status = status.to_owned();
            }

            client
                .put(key, serde_json::to_string(&scene)?, None)
                .await?;
            Ok(scene)
        } else {
            Err(EntityRepositoryError::NotFound(id.to_owned()))
        }
    }

    // async fn scenes_watch(&self) -> EntityRepositoryResult<WatchHandle<SceneEvent>> {
    //     let mut client = self.client.clone();
    //     let options = etcd_client::WatchOptions::new()
    //         .with_prefix()
    //         .with_prev_key();

    //     let (etcd_watcher, stream) = client.watch(etcd_scene_key(None), Some(options)).await?;

    //     let (etcd_handle, rx) = EtcdWatcher::start(etcd_watcher, stream);

    //     let id = WatchId::generate();
    //     self.scene_watchers
    //         .write()
    //         .unwrap()
    //         .insert(id.clone(), etcd_handle);

    //     let handle = WatchHandle::new(id, rx);

    //     // let stream = ScenesEventStream::new(event_rx);
    //     // let handle = WatchHandle::new(watcher, stream);
    //     Ok(handle)
    // }

    // async fn scenes_stop_watch(&self, watcher: WatchHandle<SceneEvent>) {
    //     let mut watch_handle = self
    //         .scene_watchers
    //         .write()
    //         .unwrap()
    //         .remove(&watcher.id)
    //         .unwrap();
    //     watch_handle.watcher.cancel().await.unwrap();
    //     watch_handle.handle.await.unwrap();
    // }

    // async fn scenes_stop_all_watchers(&self) {
    //     let ids: Vec<WatchId> = self
    //         .scene_watchers
    //         .read()
    //         .unwrap()
    //         .keys()
    //         .map(|k| k.to_owned())
    //         .collect();
    //     for id in ids {
    //         let mut watch_handle = self.scene_watchers.write().unwrap().remove(&id).unwrap();
    //         debug!("canceling watcher {}", id);
    //         watch_handle.watcher.cancel().await.unwrap();
    //         drop(watch_handle);
    //     }
    // }
}

// #[cfg(test)]
// mod tests {
//     use rstest::*;

//     use crate::core::entities::{fixtures::mock_scene, SceneEntity};

//     use super::*;

//     const DEFAULT_ETCD_HOST: &str = "localhost:2379";

//     #[fixture]
//     async fn test_client() -> etcd_client::Client {
//         etcd_client::Client::connect(&[DEFAULT_ETCD_HOST], None)
//             .await
//             .unwrap()
//     }

//     async fn add_scene(client: &mut etcd_client::Client, scene: &SceneEntity) {
//         let key = etcd_scene_key(scene.id());
//         client
//             .put(key.clone(), serde_json::to_string(scene).unwrap(), None)
//             .await
//             .unwrap();
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn create_succeeds(mock_scene: SceneEntity) -> EntityRepositoryResult<()> {
//         let client = test_client().await;
//         let repo = EtcdRepository::new(client);
//         let result = repo
//             .scenes_create(mock_scene.name(), &mock_scene.status())
//             .await;
//         println!("result: {:?}", result);
//         assert!(result.is_ok());
//         Ok(())
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn create_returns_the_scene(mock_scene: SceneEntity) -> EntityRepositoryResult<()> {
//         let client = test_client().await;
//         let repo = EtcdRepository::new(client);
//         let new_scene = repo
//             .scenes_create(mock_scene.name(), &mock_scene.status())
//             .await?;
//         assert_eq!(new_scene.name(), mock_scene.name());
//         assert_eq!(new_scene.status(), mock_scene.status());
//         Ok(())
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn create_adds_record(mock_scene: SceneEntity) {
//         let client = test_client().await;
//         let repo = EtcdRepository::new(client);
//         let new_scene = repo
//             .scenes_create(mock_scene.name(), &mock_scene.status())
//             .await
//             .unwrap();

//         let mut client = test_client().await;
//         let result = client
//             .get(etcd_scene_key(new_scene.id()), None)
//             .await
//             .unwrap();
//         assert_eq!(result.count(), 1);
//         let repo_scene: SceneEntity =
//             serde_json::from_slice(result.kvs().first().unwrap().value()).unwrap();
//         assert_eq!(repo_scene.name(), mock_scene.name());
//         assert_eq!(repo_scene.status(), mock_scene.status());
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn delete_fails_when_record_does_not_exist(mock_scene: SceneEntity) {
//         let repo = EtcdRepository::new(test_client().await);

//         let result = repo.scenes_delete(mock_scene.id()).await;
//         assert!(result.is_err());
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn delete_removes_existing_record(mock_scene: SceneEntity) {
//         let repo = EtcdRepository::new(test_client().await);
//         let mut client = test_client().await;
//         add_scene(&mut client, &mock_scene).await;

//         repo.scenes_delete(mock_scene.id()).await.unwrap();

//         assert_eq!(
//             client
//                 .get(etcd_scene_key(mock_scene.id()), None)
//                 .await
//                 .unwrap()
//                 .count(),
//             0
//         );
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn exists_returns_false_when_no_scenes_exist() {
//         let repo = EtcdRepository::new(test_client().await);

//         let query = ScenesQuery::default();

//         assert_eq!(repo.scenes_exists(query).await.unwrap(), false);
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn exists_returns_false_when_no_scene_matches(mock_scene: SceneEntity) {
//         let repo = EtcdRepository::new(test_client().await);
//         let mut client = test_client().await;
//         add_scene(&mut client, &mock_scene).await;

//         let name = format!("{}extra", mock_scene.name());
//         let query = ScenesQuery::builder().name_eq(&name).build();

//         assert_eq!(repo.scenes_exists(query).await.unwrap(), false);
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn exists_returns_true_when_scene_matches(mock_scene: SceneEntity) {
//         let repo = EtcdRepository::new(test_client().await);
//         let mut client = test_client().await;
//         add_scene(&mut client, &mock_scene).await;

//         let query = ScenesQuery::builder()
//             .name_eq(&mock_scene.name())
//             .build();

//         assert_eq!(repo.scenes_exists(query).await.unwrap(), true);
//     }
// }
