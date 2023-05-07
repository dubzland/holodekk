//! In-memory repository implementation

pub mod database;
pub use database::Database;
mod scenes;
pub use scenes::*;
mod subroutines;
pub use subroutines::*;

use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use log::debug;
use tokio::sync::broadcast::{channel, Sender};

use crate::core::scene;
use crate::entity;

/// An in-memory repository (mainly for testing)
#[derive(Debug)]
pub struct Memory {
    db: Arc<Database>,
    scene_notify_tx: RwLock<Option<Sender<scene::entity::repository::Event>>>,
}

impl Default for Memory {
    fn default() -> Self {
        let (scene_notify_tx, _scene_notify_rx) = channel(10);
        Self {
            db: Arc::new(Database::new()),
            scene_notify_tx: RwLock::new(Some(scene_notify_tx)),
        }
    }
}

impl Memory {
    /// Construct a new memory repository
    #[must_use]
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            ..Memory::default()
        }
    }
}

#[async_trait]
impl entity::Repository for Memory {
    async fn init(&self) -> entity::repository::Result<()> {
        Ok(())
    }

    async fn shutdown(&self) {
        debug!("Shutting down memory repository ...");
        if let Some(scene_notify_tx) = self.scene_notify_tx.write().unwrap().take() {
            drop(scene_notify_tx);
        }
        debug!("Shutdown complete.");
    }

    async fn subscribe_scenes(
        &self,
    ) -> entity::repository::Result<
        entity::repository::watch::Handle<scene::entity::repository::Event>,
    > {
        let id = entity::repository::watch::Id::generate();
        Ok(entity::repository::watch::Handle {
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

    use crate::core::scene::{self, entity::mock_entity as mock_scene_entity};
    use crate::core::subroutine::{self, entity::mock_entity as mock_subroutine_entity};

    use super::*;

    #[rstest]
    #[test]
    fn can_add_scene(mock_scene_entity: scene::Entity) {
        let db = Database::new();

        let result = db.scenes().add(mock_scene_entity);
        assert!(result.is_ok())
    }

    #[rstest]
    #[test]
    fn can_get_scene(mock_scene_entity: scene::Entity) {
        let db = Database::new();
        db.scenes().add(mock_scene_entity.clone()).unwrap();

        let new_scene = db.scenes().get(&mock_scene_entity.id).unwrap();
        assert_eq!(new_scene.id, mock_scene_entity.id);
    }

    #[rstest]
    #[test]
    fn can_get_subroutine(
        mock_subroutine_entity: subroutine::Entity,
    ) -> entity::repository::Result<()> {
        let db = Database::new();
        db.subroutines().add(mock_subroutine_entity.clone())?;

        let new_sub = db.subroutines().get(&mock_subroutine_entity.id)?;
        assert_eq!(new_sub.id, mock_subroutine_entity.id);
        Ok(())
    }

    #[rstest]
    #[test]
    fn can_add_subroutine(mock_subroutine_entity: subroutine::Entity) {
        let db = Database::new();

        let result = db.subroutines().add(mock_subroutine_entity.to_owned());
        assert!(result.is_ok())
    }
}
