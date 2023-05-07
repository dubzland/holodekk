use async_trait::async_trait;
use etcd_client::GetOptions;
use timestamps::Timestamps;

use crate::core::scene::{
    self,
    entity::{
        repository::{Event, Query},
        Id, Name, Status,
    },
    Entity,
};
use crate::entity::repository::{Error, Query as RepositoryQuery, Result};

use super::{scene_key, Etcd};

impl From<etcd_client::Event> for Event {
    fn from(event: etcd_client::Event) -> Self {
        if let Some(kv) = event.kv() {
            match event.event_type() {
                etcd_client::EventType::Put => {
                    let value = kv.value_str().unwrap();
                    let current: Entity = serde_json::from_str(value).unwrap();
                    if let Some(prev_kv) = event.prev_kv() {
                        let prev_value = prev_kv.value_str().unwrap();
                        let orig: Entity = serde_json::from_str(prev_value).unwrap();

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
                    let prev: Entity = serde_json::from_str(prev_value).unwrap();
                    Self::Delete { scene: prev }
                }
            }
        } else {
            Self::Unknown
        }
    }
}

#[async_trait]
impl scene::entity::Repository for Etcd {
    async fn scenes_create(&self, mut scene: Entity) -> Result<Entity> {
        match self.scenes_get(&scene.id).await {
            Err(Error::NotFound(_)) => {
                scene.created();
                scene.updated();
                let serialized = serde_json::to_string(&scene)?;
                let key = scene_key(Some(&scene.id));
                let mut client = self.client.read().unwrap().clone().unwrap();
                client.put(key, serialized, None).await?;
                Ok(scene)
            }
            Ok(_) => Err(Error::Conflict(format!(
                "Scene already exists with id {}",
                scene.id
            ))),
            Err(err) => Err(err),
        }
    }

    async fn scenes_delete(&self, id: &Id) -> Result<()> {
        let mut client = self.client.read().unwrap().clone().unwrap();
        let key = scene_key(Some(id));
        let result = client
            .get(key.clone(), Some(GetOptions::new().with_count_only()))
            .await?;
        if result.count() == 0 {
            Err(Error::NotFound(id.clone()))
        } else {
            client.delete(key, None).await?;
            Ok(())
        }
    }

    async fn scenes_exists<'a>(&self, query: Query<'a>) -> Result<bool> {
        let mut client = self.client.read().unwrap().clone().unwrap();
        let key = scene_key(None);
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
                .any(|v| match serde_json::from_slice::<Entity>(v.value()) {
                    Ok(scene) => query.matches(&scene),
                    Err(_) => false,
                });

        Ok(exists)
    }

    async fn scenes_find<'a>(&self, query: Query<'a>) -> Result<Vec<Entity>> {
        let mut client = self.client.read().unwrap().clone().unwrap();
        let key = scene_key(None);
        let result = client
            .get(key, Some(GetOptions::new().with_prefix()))
            .await?;

        let scenes = result
            .kvs()
            .iter()
            .filter_map(|v| match serde_json::from_slice::<Entity>(v.value()) {
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

    async fn scenes_get(&self, id: &Id) -> Result<Entity> {
        let mut client = self.client.read().unwrap().clone().unwrap();
        let key = scene_key(Some(id));
        let result = client.get(key, None).await?;

        if result.count() != 1 {
            Err(Error::NotFound(id.clone()))
        } else if let Some(kv) = result.kvs().first() {
            let scene: Entity = serde_json::from_slice(kv.value())?;
            Ok(scene)
        } else {
            Err(Error::NotFound(id.clone()))
        }
    }

    async fn scenes_update(
        &self,
        id: &Id,
        name: Option<Name>,
        status: Option<Status>,
    ) -> Result<Entity> {
        let mut client = self.client.read().unwrap().clone().unwrap();
        let key = scene_key(Some(id));
        let result = client.get(key.clone(), None).await?;

        if let Some(kv) = result.kvs().first() {
            let mut scene: Entity = serde_json::from_slice(kv.value())?;
            if let Some(name) = name {
                scene.name = name;
            }
            if let Some(status) = status {
                scene.status = status;
            }

            client
                .put(key, serde_json::to_string(&scene)?, None)
                .await?;
            Ok(scene)
        } else {
            Err(Error::NotFound(id.clone()))
        }
    }
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
//         let key = scene_key(scene.id());
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
//             .get(scene_key(new_scene.id()), None)
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
//                 .get(scene_key(mock_scene.id()), None)
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
