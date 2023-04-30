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

use crate::core::repositories::{Error, Repository, SceneEvent, WatchHandle, WatchId};

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

    use crate::core::entities::{fixtures::mock_scene, SceneEntity};

    use super::*;

    #[rstest]
    #[test]
    fn can_add_scene(mock_scene: SceneEntity) {
        let db = MemoryDatabase::new();

        let result = db.scenes().add(mock_scene);
        assert!(result.is_ok())
    }

    #[rstest]
    #[test]
    fn can_get_scene(mock_scene: SceneEntity) {
        let db = MemoryDatabase::new();
        db.scenes().add(mock_scene.clone()).unwrap();

        let new_scene = db.scenes().get(&mock_scene.id).unwrap();
        assert_eq!(new_scene.id, mock_scene.id);
    }

    // #[rstest]
    // #[test]
    // fn can_get_subroutine(subroutine: SubroutineEntity) -> Result<()> {
    //     let db = MemoryDatabase::new();
    //     // let key = subroutine_key(&subroutine.id);
    //     db.subroutines().add(subroutine.clone())?;

    //     let new_sub = db.subroutines().get(&subroutine.id())?;
    //     assert_eq!(new_sub.id(), subroutine.id());
    //     Ok(())
    // }

    // #[rstest]
    // #[test]
    // fn can_add_subroutine(subroutine: SubroutineEntity) {
    //     let db = MemoryDatabase::new();

    //     let result = db.subroutines().add(subroutine.to_owned());
    //     assert!(result.is_ok())
    // }

    // #[rstest]
    // #[test]
    // fn can_get_subroutine(subroutine: SubroutineEntity) -> Result<()> {
    //     let db = MemoryDatabase::new();
    //     // let key = subroutine_key(&subroutine.id);
    //     db.subroutines().add(subroutine.clone())?;

    //     let new_sub = db.subroutines().get(&subroutine.id())?;
    //     assert_eq!(new_sub.id(), subroutine.id());
    //     Ok(())
    // }
}
