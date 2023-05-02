mod data;
pub use data::*;
mod scenes;
pub use scenes::*;
mod subroutines;
pub use subroutines::*;

use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use log::debug;
use tokio::sync::broadcast::{channel, Sender};

use crate::core::entities::{
    repository::{Error, Repository, WatchHandle, WatchId},
    SceneEvent,
};

#[derive(Debug)]
pub struct MemoryRepository {
    db: Arc<MemoryDatabase>,
    scene_notify_tx: RwLock<Option<Sender<SceneEvent>>>,
}

impl Default for MemoryRepository {
    fn default() -> Self {
        let (scene_notify_tx, _scene_notify_rx) = channel(10);
        Self {
            db: Arc::new(MemoryDatabase::new()),
            scene_notify_tx: RwLock::new(Some(scene_notify_tx)),
        }
    }
}

impl MemoryRepository {
    pub fn new(db: Arc<MemoryDatabase>) -> Self {
        Self {
            db,
            ..Default::default()
        }
    }
}

#[async_trait]
impl Repository for MemoryRepository {
    async fn init(&self) -> std::result::Result<(), Error> {
        Ok(())
    }

    async fn shutdown(&self) {
        debug!("Shutting down memory repository ...");
        if let Some(scene_notify_tx) = self.scene_notify_tx.write().unwrap().take() {
            drop(scene_notify_tx);
        }
        debug!("Shutdown complete.");
    }

    async fn subscribe_scenes(&self) -> Result<WatchHandle<SceneEvent>> {
        let id = WatchId::generate();
        Ok(WatchHandle {
            id,
            rx: self
                .scene_notify_tx
                .read()
                .unwrap()
                .clone()
                .unwrap()
                .subscribe(),
        })
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use crate::core::entities::{
        fixtures::{mock_scene_entity, mock_subroutine_entity},
        SceneEntity, SubroutineEntity,
    };

    use super::*;

    #[rstest]
    #[test]
    fn can_add_scene(mock_scene_entity: SceneEntity) {
        let db = MemoryDatabase::new();

        let result = db.scenes().add(mock_scene_entity);
        assert!(result.is_ok())
    }

    #[rstest]
    #[test]
    fn can_get_scene(mock_scene_entity: SceneEntity) {
        let db = MemoryDatabase::new();
        db.scenes().add(mock_scene_entity.clone()).unwrap();

        let new_scene = db.scenes().get(&mock_scene_entity.id).unwrap();
        assert_eq!(new_scene.id, mock_scene_entity.id);
    }

    #[rstest]
    #[test]
    fn can_get_subroutine(mock_subroutine_entity: SubroutineEntity) -> Result<()> {
        let db = MemoryDatabase::new();
        db.subroutines().add(mock_subroutine_entity.clone())?;

        let new_sub = db.subroutines().get(&mock_subroutine_entity.id)?;
        assert_eq!(new_sub.id, mock_subroutine_entity.id);
        Ok(())
    }

    #[rstest]
    #[test]
    fn can_add_subroutine(mock_subroutine_entity: SubroutineEntity) {
        let db = MemoryDatabase::new();

        let result = db.subroutines().add(mock_subroutine_entity.to_owned());
        assert!(result.is_ok())
    }
}
