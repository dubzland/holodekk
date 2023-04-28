use async_trait::async_trait;
use timestamps::Timestamps;

use crate::core::{
    entities::{SceneEntity, SceneEntityId, SceneName},
    enums::SceneStatus,
    repositories::{ScenesQuery, ScenesRepository},
};
use crate::repositories::{RepositoryQuery, Result};

use super::MemoryRepository;

#[async_trait]
impl ScenesRepository for MemoryRepository {
    async fn scenes_create(&self, mut scene: SceneEntity) -> Result<SceneEntity> {
        scene.created();
        scene.updated();
        self.db.scenes().add(scene.clone())?;
        Ok(scene)
    }

    async fn scenes_delete(&self, id: &SceneEntityId) -> Result<()> {
        self.db.scenes().delete(id)
    }

    async fn scenes_exists<'a>(&self, query: ScenesQuery<'a>) -> Result<bool> {
        Ok(self.db.scenes().all().iter().any(|p| query.matches(p)))
    }

    async fn scenes_find<'a>(&self, query: ScenesQuery<'a>) -> Result<Vec<SceneEntity>> {
        let scenes = self
            .db
            .scenes()
            .all()
            .into_iter()
            .filter(|p| query.matches(p))
            .collect();
        Ok(scenes)
    }

    async fn scenes_get(&self, id: &SceneEntityId) -> Result<SceneEntity> {
        self.db.scenes().get(id)
    }

    async fn scenes_update(
        &self,
        id: &SceneEntityId,
        name: Option<SceneName>,
        status: Option<SceneStatus>,
    ) -> Result<SceneEntity> {
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

    use crate::core::entities::{fixtures::mock_scene, SceneEntity};
    use crate::repositories::memory::MemoryDatabase;
    use crate::repositories::RepositoryError;

    use super::*;

    #[fixture]
    fn db() -> Arc<MemoryDatabase> {
        Arc::new(MemoryDatabase::new())
    }

    #[rstest]
    #[tokio::test]
    async fn create_succeeds(db: Arc<MemoryDatabase>, mock_scene: SceneEntity) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let result = repo.scenes_create(mock_scene).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_returns_the_scene(
        db: Arc<MemoryDatabase>,
        mock_scene: SceneEntity,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let new_scene = repo.scenes_create(mock_scene.clone()).await?;
        assert_eq!(new_scene.name, mock_scene.name);
        assert_eq!(new_scene.status, mock_scene.status);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_adds_record(db: Arc<MemoryDatabase>, mock_scene: SceneEntity) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let new_scene = repo.scenes_create(mock_scene).await?;
        let db_scene = db.scenes().get(&new_scene.id)?;
        assert_eq!(new_scene.id, db_scene.id);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_fails_for_nonexistent_scene(db: Arc<MemoryDatabase>) -> Result<()> {
        let scene_id = SceneEntityId::generate();
        let repo = MemoryRepository::new(db.clone());
        let res = repo.scenes_delete(&scene_id).await;
        assert!(matches!(res.unwrap_err(), RepositoryError::NotFound(..)));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_removes_the_record(
        db: Arc<MemoryDatabase>,
        mock_scene: SceneEntity,
    ) -> Result<()> {
        db.scenes().add(mock_scene.clone())?;
        let repo = MemoryRepository::new(db.clone());
        repo.scenes_delete(&mock_scene.id).await?;

        assert!(!db.scenes().exists(&mock_scene.id)?);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_true_for_existing_scene(
        db: Arc<MemoryDatabase>,
        mock_scene: SceneEntity,
    ) -> Result<()> {
        db.scenes().add(mock_scene.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let query = ScenesQuery::builder().name_eq(&mock_scene.name).build();
        assert!(repo.scenes_exists(query).await?);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_false_for_nonexistent_scene(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let name: SceneName = "nonexistent".into();
        let query = ScenesQuery::builder().name_eq(&name).build();
        assert!(!repo.scenes_exists(query).await?);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_nothing_when_no_records(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        assert!(repo.scenes_find(ScenesQuery::default()).await?.is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_nothing_when_no_matches(
        db: Arc<MemoryDatabase>,
        mock_scene: SceneEntity,
    ) -> Result<()> {
        db.scenes().add(mock_scene.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let name: SceneName = "nonexistent".into();
        assert!(repo
            .scenes_find(ScenesQuery::builder().name_eq(&name).build())
            .await?
            .is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_matches(db: Arc<MemoryDatabase>, mock_scene: SceneEntity) -> Result<()> {
        db.scenes().add(mock_scene.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let res = repo
            .scenes_find(ScenesQuery::builder().name_eq(&mock_scene.name).build())
            .await?;
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], mock_scene);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_fails_when_scene_does_not_exist(
        db: Arc<MemoryDatabase>,
        mock_scene: SceneEntity,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let res = repo.scenes_get(&mock_scene.id).await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), RepositoryError::NotFound(..)));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_returns_scene(db: Arc<MemoryDatabase>, mock_scene: SceneEntity) -> Result<()> {
        db.scenes().add(mock_scene.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let s = repo.scenes_get(&mock_scene.id).await?;
        assert_eq!(s, mock_scene);
        Ok(())
    }
}
