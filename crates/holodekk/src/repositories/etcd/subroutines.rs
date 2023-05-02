use async_trait::async_trait;
use etcd_client::GetOptions;
use timestamps::Timestamps;

use crate::core::{
    entities::{
        repository::{Error, RepositoryQuery, Result},
        EntityId, SubroutineEntity, SubroutineEntityId, SubroutineEvent, SubroutinesQuery,
        SubroutinesRepository,
    },
    enums::SubroutineStatus,
};

use super::{etcd_subroutine_key, EtcdRepository};

impl From<etcd_client::Event> for SubroutineEvent {
    fn from(event: etcd_client::Event) -> Self {
        if let Some(kv) = event.kv() {
            match event.event_type() {
                etcd_client::EventType::Put => {
                    let value = kv.value_str().unwrap();
                    let current: SubroutineEntity = serde_json::from_str(value).unwrap();
                    if let Some(prev_kv) = event.prev_kv() {
                        let prev_value = prev_kv.value_str().unwrap();
                        let orig: SubroutineEntity = serde_json::from_str(prev_value).unwrap();

                        Self::Update {
                            subroutine: current,
                            orig,
                        }
                    } else {
                        Self::Insert {
                            subroutine: current,
                        }
                    }
                }
                etcd_client::EventType::Delete => {
                    let prev_value = event.prev_kv().unwrap().value_str().unwrap();
                    let prev: SubroutineEntity = serde_json::from_str(prev_value).unwrap();
                    Self::Delete { subroutine: prev }
                }
            }
        } else {
            Self::Unknown
        }
    }
}

#[async_trait]
impl SubroutinesRepository for EtcdRepository {
    async fn subroutines_create(
        &self,
        mut subroutine: SubroutineEntity,
    ) -> Result<SubroutineEntity> {
        match self.subroutines_get(&subroutine.id).await {
            Err(Error::NotFound(_)) => {
                subroutine.created();
                subroutine.updated();
                let serialized = serde_json::to_string(&subroutine)?;
                let key = etcd_subroutine_key(Some(&subroutine.id));
                let mut client = self.client.read().unwrap().clone().unwrap();
                client.put(key, serialized, None).await?;
                Ok(subroutine)
            }
            Ok(_) => Err(Error::Conflict(format!(
                "Subroutine already exists with id {}",
                subroutine.id
            ))),
            Err(err) => Err(err),
        }
    }

    async fn subroutines_delete(&self, id: &EntityId) -> Result<()> {
        let mut client = self.client.read().unwrap().clone().unwrap();
        let key = etcd_subroutine_key(Some(id));
        let result = client
            .get(key.clone(), Some(GetOptions::new().with_count_only()))
            .await?;
        if result.count() == 0 {
            Err(Error::NotFound(id.to_owned()))
        } else {
            client.delete(key, None).await?;
            Ok(())
        }
    }

    async fn subroutines_exists<'a>(&self, query: SubroutinesQuery<'a>) -> Result<bool> {
        let mut client = self.client.read().unwrap().clone().unwrap();
        let key = etcd_subroutine_key(None);
        let result = client
            .get(key, Some(GetOptions::new().with_prefix()))
            .await?;

        if result.count() == 0 {
            return Ok(false);
        }

        let exists = result.kvs().iter().any(|v| {
            match serde_json::from_slice::<SubroutineEntity>(v.value()) {
                Ok(subroutine) => query.matches(&subroutine),
                Err(_) => false,
            }
        });

        Ok(exists)
    }

    async fn subroutines_find<'a>(
        &self,
        query: SubroutinesQuery<'a>,
    ) -> Result<Vec<SubroutineEntity>> {
        let mut client = self.client.read().unwrap().clone().unwrap();
        let key = etcd_subroutine_key(None);
        let result = client
            .get(key, Some(GetOptions::new().with_prefix()))
            .await?;

        let subroutines = result
            .kvs()
            .iter()
            .filter_map(
                |v| match serde_json::from_slice::<SubroutineEntity>(v.value()) {
                    Ok(subroutine) => {
                        if query.matches(&subroutine) {
                            Some(subroutine)
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                },
            )
            .collect();

        Ok(subroutines)
    }

    async fn subroutines_get(&self, id: &EntityId) -> Result<SubroutineEntity> {
        let mut client = self.client.read().unwrap().clone().unwrap();
        let key = etcd_subroutine_key(Some(id));
        let result = client.get(key, None).await?;

        if result.count() != 1 {
            Err(Error::NotFound(id.to_owned()))
        } else if let Some(kv) = result.kvs().first() {
            let subroutine: SubroutineEntity = serde_json::from_slice(kv.value())?;
            Ok(subroutine)
        } else {
            Err(Error::NotFound(id.to_owned()))
        }
    }

    async fn subroutines_update(
        &self,
        id: &SubroutineEntityId,
        status: Option<SubroutineStatus>,
    ) -> Result<SubroutineEntity> {
        let mut client = self.client.read().unwrap().clone().unwrap();
        let key = etcd_subroutine_key(Some(id));
        let result = client.get(key.clone(), None).await?;

        if let Some(kv) = result.kvs().first() {
            let mut subroutine: SubroutineEntity = serde_json::from_slice(kv.value())?;
            if let Some(status) = status {
                subroutine.status = status.to_owned();
            }

            client
                .put(key, serde_json::to_string(&subroutine)?, None)
                .await?;
            Ok(subroutine)
        } else {
            Err(Error::NotFound(id.to_owned()))
        }
    }

    // async fn subroutines_watch(&self) -> Result<WatchHandle<SubroutineEvent>> {
    //     let mut client = self.client.clone();
    //     let options = etcd_client::WatchOptions::new()
    //         .with_prefix()
    //         .with_prev_key();

    //     let (etcd_watcher, stream) = client.watch(etcd_subroutine_key(None), Some(options)).await?;

    //     let (etcd_handle, rx) = EtcdWatcher::start(etcd_watcher, stream);

    //     let id = WatchId::generate();
    //     self.subroutine_watchers
    //         .write()
    //         .unwrap()
    //         .insert(id.clone(), etcd_handle);

    //     let handle = WatchHandle::new(id, rx);

    //     // let stream = SubroutinesEventStream::new(event_rx);
    //     // let handle = WatchHandle::new(watcher, stream);
    //     Ok(handle)
    // }

    // async fn subroutines_stop_watch(&self, watcher: WatchHandle<SubroutineEvent>) {
    //     let mut watch_handle = self
    //         .subroutine_watchers
    //         .write()
    //         .unwrap()
    //         .remove(&watcher.id)
    //         .unwrap();
    //     watch_handle.watcher.cancel().await.unwrap();
    //     watch_handle.handle.await.unwrap();
    // }

    // async fn subroutines_stop_all_watchers(&self) {
    //     let ids: Vec<WatchId> = self
    //         .subroutine_watchers
    //         .read()
    //         .unwrap()
    //         .keys()
    //         .map(|k| k.to_owned())
    //         .collect();
    //     for id in ids {
    //         let mut watch_handle = self.subroutine_watchers.write().unwrap().remove(&id).unwrap();
    //         debug!("canceling watcher {}", id);
    //         watch_handle.watcher.cancel().await.unwrap();
    //         drop(watch_handle);
    //     }
    // }
}

// #[cfg(test)]
// mod tests {
//     use rstest::*;

//     use crate::core::entities::{fixtures::mock_subroutine, SubroutineEntity};

//     use super::*;

//     const DEFAULT_ETCD_HOST: &str = "localhost:2379";

//     #[fixture]
//     async fn test_client() -> etcd_client::Client {
//         etcd_client::Client::connect(&[DEFAULT_ETCD_HOST], None)
//             .await
//             .unwrap()
//     }

//     async fn add_subroutine(client: &mut etcd_client::Client, subroutine: &SubroutineEntity) {
//         let key = etcd_subroutine_key(subroutine.id());
//         client
//             .put(key.clone(), serde_json::to_string(subroutine).unwrap(), None)
//             .await
//             .unwrap();
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn create_succeeds(mock_subroutine: SubroutineEntity) -> Result<()> {
//         let client = test_client().await;
//         let repo = EtcdRepository::new(client);
//         let result = repo
//             .subroutines_create(mock_subroutine.name(), &mock_subroutine.status())
//             .await;
//         println!("result: {:?}", result);
//         assert!(result.is_ok());
//         Ok(())
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn create_returns_the_subroutine(mock_subroutine: SubroutineEntity) -> Result<()> {
//         let client = test_client().await;
//         let repo = EtcdRepository::new(client);
//         let new_subroutine = repo
//             .subroutines_create(mock_subroutine.name(), &mock_subroutine.status())
//             .await?;
//         assert_eq!(new_subroutine.name(), mock_subroutine.name());
//         assert_eq!(new_subroutine.status(), mock_subroutine.status());
//         Ok(())
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn create_adds_record(mock_subroutine: SubroutineEntity) {
//         let client = test_client().await;
//         let repo = EtcdRepository::new(client);
//         let new_subroutine = repo
//             .subroutines_create(mock_subroutine.name(), &mock_subroutine.status())
//             .await
//             .unwrap();

//         let mut client = test_client().await;
//         let result = client
//             .get(etcd_subroutine_key(new_subroutine.id()), None)
//             .await
//             .unwrap();
//         assert_eq!(result.count(), 1);
//         let repo_subroutine: SubroutineEntity =
//             serde_json::from_slice(result.kvs().first().unwrap().value()).unwrap();
//         assert_eq!(repo_subroutine.name(), mock_subroutine.name());
//         assert_eq!(repo_subroutine.status(), mock_subroutine.status());
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn delete_fails_when_record_does_not_exist(mock_subroutine: SubroutineEntity) {
//         let repo = EtcdRepository::new(test_client().await);

//         let result = repo.subroutines_delete(mock_subroutine.id()).await;
//         assert!(result.is_err());
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn delete_removes_existing_record(mock_subroutine: SubroutineEntity) {
//         let repo = EtcdRepository::new(test_client().await);
//         let mut client = test_client().await;
//         add_subroutine(&mut client, &mock_subroutine).await;

//         repo.subroutines_delete(mock_subroutine.id()).await.unwrap();

//         assert_eq!(
//             client
//                 .get(etcd_subroutine_key(mock_subroutine.id()), None)
//                 .await
//                 .unwrap()
//                 .count(),
//             0
//         );
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn exists_returns_false_when_no_subroutines_exist() {
//         let repo = EtcdRepository::new(test_client().await);

//         let query = SubroutinesQuery::default();

//         assert_eq!(repo.subroutines_exists(query).await.unwrap(), false);
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn exists_returns_false_when_no_subroutine_matches(mock_subroutine: SubroutineEntity) {
//         let repo = EtcdRepository::new(test_client().await);
//         let mut client = test_client().await;
//         add_subroutine(&mut client, &mock_subroutine).await;

//         let name = format!("{}extra", mock_subroutine.name());
//         let query = SubroutinesQuery::builder().name_eq(&name).build();

//         assert_eq!(repo.subroutines_exists(query).await.unwrap(), false);
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn exists_returns_true_when_subroutine_matches(mock_subroutine: SubroutineEntity) {
//         let repo = EtcdRepository::new(test_client().await);
//         let mut client = test_client().await;
//         add_subroutine(&mut client, &mock_subroutine).await;

//         let query = SubroutinesQuery::builder()
//             .name_eq(&mock_subroutine.name())
//             .build();

//         assert_eq!(repo.subroutines_exists(query).await.unwrap(), true);
//     }
// }
