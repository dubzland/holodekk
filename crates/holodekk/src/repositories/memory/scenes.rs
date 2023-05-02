use async_trait::async_trait;
use log::warn;
use timestamps::Timestamps;

use crate::core::entities::{
    EntityRepositoryQuery, EntityRepositoryResult, SceneEntity, SceneEntityId,
    SceneEntityRepository, SceneEntityRepositoryEvent, SceneEntityRepositoryQuery, SceneName,
};
use crate::enums::SceneStatus;

use super::MemoryRepository;

impl MemoryRepository {
    pub async fn broadcast_scene_notification(&mut self, msg: SceneEntityRepositoryEvent) {
        if let Some(tx) = self.scene_notify_tx.read().unwrap().as_ref() {
            if let Err(err) = tx.send(msg) {
                warn!("Error broadcasting scene repository event: {}", err);
            }
        }
    }

    pub async fn notify_scene_insert(&mut self, scene: &SceneEntity) {
        self.broadcast_scene_notification(SceneEntityRepositoryEvent::Insert {
            scene: scene.to_owned(),
        })
        .await;
    }

    pub async fn notify_scene_update(&mut self, scene: &SceneEntity, orig: &SceneEntity) {
        self.broadcast_scene_notification(SceneEntityRepositoryEvent::Update {
            scene: scene.to_owned(),
            orig: orig.to_owned(),
        })
        .await;
    }

    pub async fn notify_scene_delete(&mut self, scene: &SceneEntity) {
        self.broadcast_scene_notification(SceneEntityRepositoryEvent::Delete {
            scene: scene.to_owned(),
        })
        .await;
    }
}

#[async_trait]
impl SceneEntityRepository for MemoryRepository {
    async fn scenes_create(&self, mut scene: SceneEntity) -> EntityRepositoryResult<SceneEntity> {
        scene.created();
        scene.updated();
        self.db.scenes().add(scene.clone())?;

        Ok(scene)
    }

    async fn scenes_delete(&self, id: &SceneEntityId) -> EntityRepositoryResult<()> {
        self.db.scenes().delete(id)
    }

    async fn scenes_exists<'a>(
        &self,
        query: SceneEntityRepositoryQuery<'a>,
    ) -> EntityRepositoryResult<bool> {
        Ok(self.db.scenes().all().iter().any(|p| query.matches(p)))
    }

    async fn scenes_find<'a>(
        &self,
        query: SceneEntityRepositoryQuery<'a>,
    ) -> EntityRepositoryResult<Vec<SceneEntity>> {
        let scenes = self
            .db
            .scenes()
            .all()
            .into_iter()
            .filter(|p| query.matches(p))
            .collect();
        Ok(scenes)
    }

    async fn scenes_get(&self, id: &SceneEntityId) -> EntityRepositoryResult<SceneEntity> {
        self.db.scenes().get(id)
    }

    async fn scenes_update(
        &self,
        id: &SceneEntityId,
        name: Option<SceneName>,
        status: Option<SceneStatus>,
    ) -> EntityRepositoryResult<SceneEntity> {
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

    use crate::core::entities::{
        fixtures::mock_scene_entity, EntityRepositoryError, EntityRepositoryResult, SceneEntity,
    };
    use crate::repositories::memory::MemoryDatabase;

    use super::*;

    #[fixture]
    fn db() -> Arc<MemoryDatabase> {
        Arc::new(MemoryDatabase::new())
    }

    #[rstest]
    #[tokio::test]
    async fn create_succeeds(
        db: Arc<MemoryDatabase>,
        mock_scene_entity: SceneEntity,
    ) -> EntityRepositoryResult<()> {
        let repo = MemoryRepository::new(db.clone());
        let result = repo.scenes_create(mock_scene_entity).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_returns_the_scene(
        db: Arc<MemoryDatabase>,
        mock_scene_entity: SceneEntity,
    ) -> EntityRepositoryResult<()> {
        let repo = MemoryRepository::new(db.clone());
        let new_scene = repo.scenes_create(mock_scene_entity.clone()).await?;
        assert_eq!(new_scene.name, mock_scene_entity.name);
        assert_eq!(new_scene.status, mock_scene_entity.status);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_adds_record(
        db: Arc<MemoryDatabase>,
        mock_scene_entity: SceneEntity,
    ) -> EntityRepositoryResult<()> {
        let repo = MemoryRepository::new(db.clone());
        let new_scene = repo.scenes_create(mock_scene_entity).await?;
        let db_scene = db.scenes().get(&new_scene.id)?;
        assert_eq!(new_scene.id, db_scene.id);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_fails_for_nonexistent_scene(
        db: Arc<MemoryDatabase>,
    ) -> EntityRepositoryResult<()> {
        let scene_id = SceneEntityId::generate();
        let repo = MemoryRepository::new(db.clone());
        let res = repo.scenes_delete(&scene_id).await;
        assert!(matches!(
            res.unwrap_err(),
            EntityRepositoryError::NotFound(..)
        ));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_removes_the_record(
        db: Arc<MemoryDatabase>,
        mock_scene_entity: SceneEntity,
    ) -> EntityRepositoryResult<()> {
        db.scenes().add(mock_scene_entity.clone())?;
        let repo = MemoryRepository::new(db.clone());
        repo.scenes_delete(&mock_scene_entity.id).await?;

        assert!(!db.scenes().exists(&mock_scene_entity.id)?);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_true_for_existing_scene(
        db: Arc<MemoryDatabase>,
        mock_scene_entity: SceneEntity,
    ) -> EntityRepositoryResult<()> {
        db.scenes().add(mock_scene_entity.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let query = SceneEntityRepositoryQuery::builder()
            .name_eq(&mock_scene_entity.name)
            .build();
        assert!(repo.scenes_exists(query).await?);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_false_for_nonexistent_scene(
        db: Arc<MemoryDatabase>,
    ) -> EntityRepositoryResult<()> {
        let repo = MemoryRepository::new(db.clone());
        let name: SceneName = "nonexistent".into();
        let query = SceneEntityRepositoryQuery::builder().name_eq(&name).build();
        assert!(!repo.scenes_exists(query).await?);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_nothing_when_no_records(
        db: Arc<MemoryDatabase>,
    ) -> EntityRepositoryResult<()> {
        let repo = MemoryRepository::new(db.clone());
        assert!(repo
            .scenes_find(SceneEntityRepositoryQuery::default())
            .await?
            .is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_nothing_when_no_matches(
        db: Arc<MemoryDatabase>,
        mock_scene_entity: SceneEntity,
    ) -> EntityRepositoryResult<()> {
        db.scenes().add(mock_scene_entity.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let name: SceneName = "nonexistent".into();
        assert!(repo
            .scenes_find(SceneEntityRepositoryQuery::builder().name_eq(&name).build())
            .await?
            .is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_matches(
        db: Arc<MemoryDatabase>,
        mock_scene_entity: SceneEntity,
    ) -> EntityRepositoryResult<()> {
        db.scenes().add(mock_scene_entity.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let res = repo
            .scenes_find(
                SceneEntityRepositoryQuery::builder()
                    .name_eq(&mock_scene_entity.name)
                    .build(),
            )
            .await?;
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], mock_scene_entity);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_fails_when_scene_does_not_exist(
        db: Arc<MemoryDatabase>,
        mock_scene_entity: SceneEntity,
    ) -> EntityRepositoryResult<()> {
        let repo = MemoryRepository::new(db.clone());
        let res = repo.scenes_get(&mock_scene_entity.id).await;
        assert!(res.is_err());
        assert!(matches!(
            res.unwrap_err(),
            EntityRepositoryError::NotFound(..)
        ));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_returns_scene(
        db: Arc<MemoryDatabase>,
        mock_scene_entity: SceneEntity,
    ) -> EntityRepositoryResult<()> {
        db.scenes().add(mock_scene_entity.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let s = repo.scenes_get(&mock_scene_entity.id).await?;
        assert_eq!(s, mock_scene_entity);
        Ok(())
    }
}
