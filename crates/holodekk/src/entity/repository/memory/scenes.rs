use async_trait::async_trait;
use log::warn;
use timestamps::Timestamps;

use crate::core::scene::{
    entity::{
        repository::{Event, Query},
        Id, Name, Repository, Status,
    },
    Entity,
};
use crate::entity::repository::{Query as RepositoryQuery, Result};

use super::Memory;

impl Memory {
    /// Notifies all subscribed clients of the supplied event
    pub fn broadcast_scene_notification(&mut self, msg: Event) {
        match self.scene_notify_tx.read() {
            Ok(guard) => {
                if let Some(tx) = guard.as_ref() {
                    if let Err(err) = tx.send(msg) {
                        warn!("Error broadcasting scene repository event: {}", err);
                    }
                }
            }
            Err(err) => {
                warn!("Failed to acquire lock on notification tx: {err}");
            }
        }
    }

    /// Broadcasts an insert event
    pub fn notify_scene_insert(&mut self, scene: &Entity) {
        self.broadcast_scene_notification(Event::Insert {
            scene: scene.clone(),
        });
    }

    /// Broadcasts an update event
    pub fn notify_scene_update(&mut self, scene: &Entity, orig: &Entity) {
        self.broadcast_scene_notification(Event::Update {
            scene: scene.clone(),
            orig: orig.clone(),
        });
    }

    /// Broadcasts a delete event
    pub fn notify_scene_delete(&mut self, scene: &Entity) {
        self.broadcast_scene_notification(Event::Delete {
            scene: scene.clone(),
        });
    }
}

#[async_trait]
impl Repository for Memory {
    async fn scenes_create(&self, mut scene: Entity) -> Result<Entity> {
        scene.created();
        scene.updated();
        self.db.scenes().add(scene.clone())?;

        Ok(scene)
    }

    async fn scenes_delete(&self, id: &Id) -> Result<()> {
        self.db.scenes().delete(id)
    }

    async fn scenes_exists<'a>(&self, query: Query<'a>) -> Result<bool> {
        Ok(self.db.scenes().all().iter().any(|p| query.matches(p)))
    }

    async fn scenes_find<'a>(&self, query: Query<'a>) -> Result<Vec<Entity>> {
        let scenes = self
            .db
            .scenes()
            .all()
            .into_iter()
            .filter(|p| query.matches(p))
            .collect();
        Ok(scenes)
    }

    async fn scenes_get(&self, id: &Id) -> Result<Entity> {
        self.db.scenes().get(id)
    }

    async fn scenes_update(
        &self,
        id: &Id,
        name: Option<Name>,
        status: Option<Status>,
    ) -> Result<Entity> {
        let mut scene = self.scenes_get(id).await?;
        if let Some(name) = name {
            scene.name = name;
        }
        if let Some(status) = status {
            scene.status = status;
        }
        scene.updated();
        let scene = self.db.scenes().update(scene)?;
        Ok(scene)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::core::scene::{entity::mock_entity, Entity};
    use crate::entity::repository::{memory::Database, Error, Result};

    use super::*;

    #[fixture]
    fn db() -> Arc<Database> {
        Arc::new(Database::new())
    }

    #[rstest]
    #[tokio::test]
    async fn create_succeeds(db: Arc<Database>, mock_entity: Entity) -> Result<()> {
        let repo = Memory::new(db.clone());
        let result = repo.scenes_create(mock_entity).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_returns_the_scene(db: Arc<Database>, mock_entity: Entity) -> Result<()> {
        let repo = Memory::new(db.clone());
        let new_scene = repo.scenes_create(mock_entity.clone()).await?;
        assert_eq!(new_scene.name, mock_entity.name);
        assert_eq!(new_scene.status, mock_entity.status);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_adds_record(db: Arc<Database>, mock_entity: Entity) -> Result<()> {
        let repo = Memory::new(db.clone());
        let new_scene = repo.scenes_create(mock_entity).await?;
        let db_scene = db.scenes().get(&new_scene.id)?;
        assert_eq!(new_scene.id, db_scene.id);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_fails_for_nonexistent_scene(db: Arc<Database>) -> Result<()> {
        let scene_id = Id::generate();
        let repo = Memory::new(db.clone());
        let res = repo.scenes_delete(&scene_id).await;
        assert!(matches!(res.unwrap_err(), Error::NotFound(..)));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_removes_the_record(db: Arc<Database>, mock_entity: Entity) -> Result<()> {
        db.scenes().add(mock_entity.clone())?;
        let repo = Memory::new(db.clone());
        repo.scenes_delete(&mock_entity.id).await?;

        assert!(!db.scenes().exists(&mock_entity.id)?);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_true_for_existing_scene(
        db: Arc<Database>,
        mock_entity: Entity,
    ) -> Result<()> {
        db.scenes().add(mock_entity.clone())?;
        let repo = Memory::new(db.clone());
        let query = Query::builder().name_eq(&mock_entity.name).build();
        assert!(repo.scenes_exists(query).await?);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_false_for_nonexistent_scene(db: Arc<Database>) -> Result<()> {
        let repo = Memory::new(db.clone());
        let name: Name = "nonexistent".into();
        let query = Query::builder().name_eq(&name).build();
        assert!(!repo.scenes_exists(query).await?);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_nothing_when_no_records(db: Arc<Database>) -> Result<()> {
        let repo = Memory::new(db.clone());
        assert!(repo.scenes_find(Query::default()).await?.is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_nothing_when_no_matches(
        db: Arc<Database>,
        mock_entity: Entity,
    ) -> Result<()> {
        db.scenes().add(mock_entity.clone())?;
        let repo = Memory::new(db.clone());
        let name: Name = "nonexistent".into();
        assert!(repo
            .scenes_find(Query::builder().name_eq(&name).build())
            .await?
            .is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_matches(db: Arc<Database>, mock_entity: Entity) -> Result<()> {
        db.scenes().add(mock_entity.clone())?;
        let repo = Memory::new(db.clone());
        let res = repo
            .scenes_find(Query::builder().name_eq(&mock_entity.name).build())
            .await?;
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], mock_entity);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_fails_when_scene_does_not_exist(
        db: Arc<Database>,
        mock_entity: Entity,
    ) -> Result<()> {
        let repo = Memory::new(db.clone());
        let res = repo.scenes_get(&mock_entity.id).await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotFound(..)));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_returns_scene(db: Arc<Database>, mock_entity: Entity) -> Result<()> {
        db.scenes().add(mock_entity.clone())?;
        let repo = Memory::new(db.clone());

        let s = repo.scenes_get(&mock_entity.id).await?;
        assert_eq!(s, mock_entity);
        Ok(())
    }
}
