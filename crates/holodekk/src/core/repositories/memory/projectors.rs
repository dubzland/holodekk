use async_trait::async_trait;

use crate::core::projectors::{
    entities::Projector,
    repositories::{ProjectorsQuery, ProjectorsRepository},
};
use crate::core::repositories::{RepositoryError, RepositoryQuery, Result};

use super::MemoryRepository;

#[async_trait]
impl ProjectorsRepository for MemoryRepository {
    async fn projectors_create(&self, projector: Projector) -> Result<Projector> {
        // Ensure the projector doesn't exist
        if self.db.projectors().exists(projector.id())? {
            Err(RepositoryError::Duplicate(format!(
                "Projector {} already exists",
                projector.id()
            )))
        } else {
            self.db.projectors().add(projector.clone())?;
            Ok(projector)
        }
    }

    async fn projectors_delete(&self, id: &str) -> Result<()> {
        self.db.projectors().delete(id)
    }

    async fn projectors_exists(&self, id: &str) -> Result<bool> {
        self.db.projectors().exists(id)
    }

    async fn projectors_find(&self, query: ProjectorsQuery) -> Vec<Projector> {
        let projectors = self.db.projectors().all();
        projectors
            .into_iter()
            .filter(|p| query.matches(p))
            .collect()
    }

    async fn projectors_get(&self, id: &str) -> Result<Projector> {
        self.db.projectors().get(id)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::core::projectors::{
        entities::{fixtures::projector, Projector},
        repositories::ProjectorsQuery,
    };
    use crate::core::repositories::{self, memory::MemoryDatabase};

    use super::*;

    #[fixture]
    fn db() -> Arc<MemoryDatabase> {
        Arc::new(MemoryDatabase::new())
    }

    #[rstest]
    #[tokio::test]
    async fn create_fails_when_projector_already_exists(
        db: Arc<MemoryDatabase>,
        projector: Projector,
    ) -> Result<()> {
        db.projectors().add(projector.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let result = repo.projectors_create(projector.clone()).await;
        assert!(matches!(
            result.unwrap_err(),
            repositories::RepositoryError::Duplicate(..)
        ));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_succeeds(db: Arc<MemoryDatabase>, projector: Projector) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let result = repo.projectors_create(projector.clone()).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_adds_record(db: Arc<MemoryDatabase>, projector: Projector) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        repo.projectors_create(projector.clone()).await?;
        let new_projector = db.projectors().get(&projector.id())?;
        assert_eq!(new_projector.id(), projector.id());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_returns_the_projector(
        db: Arc<MemoryDatabase>,
        projector: Projector,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let new_projector = repo.projectors_create(projector.clone()).await?;
        assert_eq!(new_projector.id(), projector.id());
        Ok(())
    }
    #[rstest]
    #[tokio::test]
    async fn delete_fails_for_nonexistent_projector(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let res = repo.projectors_delete("nonexistent").await;
        assert!(matches!(res.unwrap_err(), RepositoryError::NotFound(..)));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_removes_the_record(
        db: Arc<MemoryDatabase>,
        projector: Projector,
    ) -> Result<()> {
        db.projectors().add(projector.clone())?;
        let repo = MemoryRepository::new(db.clone());
        repo.projectors_delete(&projector.id()).await?;

        assert!(!db.projectors().exists(&projector.id())?);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_true_for_existing_projector(
        db: Arc<MemoryDatabase>,
        projector: Projector,
    ) -> Result<()> {
        db.projectors().add(projector.clone())?;
        let repo = MemoryRepository::new(db.clone());
        assert!(repo.projectors_exists(&projector.id()).await?);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_false_for_nonexistent_projector(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        assert!(!repo.projectors_exists("nonexistent").await?);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_nothing_when_no_records(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        assert!(repo
            .projectors_find(ProjectorsQuery::default())
            .await
            .is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_nothing_when_no_matches(
        db: Arc<MemoryDatabase>,
        projector: Projector,
    ) -> Result<()> {
        db.projectors().add(projector.clone())?;
        let repo = MemoryRepository::new(db.clone());
        assert!(repo
            .projectors_find(
                ProjectorsQuery::builder()
                    .fleet_eq(&format!("{}nonexistent", projector.fleet()))
                    .build()
            )
            .await
            .is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_matches(db: Arc<MemoryDatabase>, projector: Projector) -> Result<()> {
        db.projectors().add(projector.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let res = repo
            .projectors_find(
                ProjectorsQuery::builder()
                    .fleet_eq(&projector.fleet())
                    .build(),
            )
            .await;
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], projector);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_fails_when_projector_does_not_exist(
        db: Arc<MemoryDatabase>,
        projector: Projector,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let res = repo.projectors_get(&projector.id()).await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), RepositoryError::NotFound(..)));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_returns_projector(db: Arc<MemoryDatabase>, projector: Projector) -> Result<()> {
        db.projectors().add(projector.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let p = repo.projectors_get(&projector.id()).await?;
        assert_eq!(p.id(), projector.id());
        Ok(())
    }
}
