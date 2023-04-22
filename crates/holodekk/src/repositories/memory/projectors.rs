use async_trait::async_trait;

use crate::core::projectors::{
    entities::ProjectorEntity,
    repositories::{ProjectorsQuery, ProjectorsRepository},
};
use crate::repositories::{RepositoryError, RepositoryQuery, Result};

use super::MemoryRepository;

#[async_trait]
impl ProjectorsRepository for MemoryRepository {
    async fn projectors_create(&self, projector: ProjectorEntity) -> Result<ProjectorEntity> {
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

    async fn projectors_exists(&self, query: ProjectorsQuery) -> Result<bool> {
        Ok(self.db.projectors().all().iter().any(|p| query.matches(p)))
    }

    async fn projectors_find(&self, query: ProjectorsQuery) -> Result<Vec<ProjectorEntity>> {
        let projectors = self
            .db
            .projectors()
            .all()
            .into_iter()
            .filter(|p| query.matches(p))
            .collect();
        Ok(projectors)
    }

    async fn projectors_get(&self, id: &str) -> Result<ProjectorEntity> {
        self.db.projectors().get(id)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::core::projectors::{
        entities::{fixtures::projector, ProjectorEntity},
        repositories::ProjectorsQuery,
    };
    use crate::repositories::{self, memory::MemoryDatabase};

    use super::*;

    #[fixture]
    fn db() -> Arc<MemoryDatabase> {
        Arc::new(MemoryDatabase::new())
    }

    #[rstest]
    #[tokio::test]
    async fn create_fails_when_projector_already_exists(
        db: Arc<MemoryDatabase>,
        projector: ProjectorEntity,
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
    async fn create_succeeds(db: Arc<MemoryDatabase>, projector: ProjectorEntity) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let result = repo.projectors_create(projector.clone()).await;
        assert!(result.is_ok());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_adds_record(db: Arc<MemoryDatabase>, projector: ProjectorEntity) -> Result<()> {
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
        projector: ProjectorEntity,
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
        projector: ProjectorEntity,
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
        projector: ProjectorEntity,
    ) -> Result<()> {
        db.projectors().add(projector.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let query = ProjectorsQuery::builder()
            .namespace_eq(projector.namespace())
            .build();
        assert!(repo.projectors_exists(query).await?);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_false_for_nonexistent_projector(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let query = ProjectorsQuery::builder()
            .namespace_eq("nonexistent")
            .build();
        assert!(!repo.projectors_exists(query).await?);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_nothing_when_no_records(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        assert!(repo
            .projectors_find(ProjectorsQuery::default())
            .await?
            .is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_nothing_when_no_matches(
        db: Arc<MemoryDatabase>,
        projector: ProjectorEntity,
    ) -> Result<()> {
        db.projectors().add(projector.clone())?;
        let repo = MemoryRepository::new(db.clone());
        assert!(repo
            .projectors_find(
                ProjectorsQuery::builder()
                    .namespace_eq(&format!("{}nonexistent", projector.namespace()))
                    .build()
            )
            .await?
            .is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_matches(
        db: Arc<MemoryDatabase>,
        projector: ProjectorEntity,
    ) -> Result<()> {
        db.projectors().add(projector.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let res = repo
            .projectors_find(
                ProjectorsQuery::builder()
                    .namespace_eq(&projector.namespace())
                    .build(),
            )
            .await?;
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], projector);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_fails_when_projector_does_not_exist(
        db: Arc<MemoryDatabase>,
        projector: ProjectorEntity,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let res = repo.projectors_get(&projector.id()).await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), RepositoryError::NotFound(..)));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_returns_projector(
        db: Arc<MemoryDatabase>,
        projector: ProjectorEntity,
    ) -> Result<()> {
        db.projectors().add(projector.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let p = repo.projectors_get(&projector.id()).await?;
        assert_eq!(p.id(), projector.id());
        Ok(())
    }
}
