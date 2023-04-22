use async_trait::async_trait;

pub use crate::core::subroutines::{
    entities::SubroutineEntity,
    repositories::{SubroutinesQuery, SubroutinesRepository},
};
pub use crate::repositories::{RepositoryError, RepositoryQuery, Result};

pub(self) use super::MemoryRepository;

#[async_trait]
impl SubroutinesRepository for MemoryRepository {
    async fn subroutines_create(&self, subroutine: SubroutineEntity) -> Result<SubroutineEntity> {
        self.db.subroutines().add(subroutine.clone())?;
        Ok(subroutine)
    }

    async fn subroutines_delete(&self, id: &str) -> Result<()> {
        if self.db.subroutines().exists(id)? {
            self.db.subroutines().delete(id)?;
            Ok(())
        } else {
            Err(RepositoryError::NotFound(id.to_string()))
        }
    }

    async fn subroutines_exists<'a>(&self, query: &'a SubroutinesQuery<'a>) -> Result<bool> {
        Ok(self.db.subroutines().all().iter().any(|s| query.matches(s)))
    }

    async fn subroutines_find<'a>(
        &self,
        query: &'a SubroutinesQuery<'a>,
    ) -> Result<Vec<SubroutineEntity>> {
        Ok(self
            .db
            .subroutines()
            .all()
            .into_iter()
            .filter(|i| query.matches(i))
            .collect())
    }

    async fn subroutines_get(&self, id: &str) -> Result<SubroutineEntity> {
        let subroutine = self.db.subroutines().get(id)?;
        Ok(subroutine)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::{
        core::subroutines::{entities::fixtures::subroutine, repositories::SubroutinesQuery},
        repositories::{memory::MemoryDatabase, RepositoryError},
    };

    use super::*;

    #[fixture]
    fn db() -> Arc<MemoryDatabase> {
        Arc::new(MemoryDatabase::new())
    }

    #[rstest]
    #[tokio::test]
    async fn create_fails_when_subroutine_already_exists(
        db: Arc<MemoryDatabase>,
        subroutine: SubroutineEntity,
    ) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let result = repo.subroutines_create(subroutine.clone()).await;

        assert!(matches!(
            result.unwrap_err(),
            RepositoryError::Duplicate(..)
        ));

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_succeeds(db: Arc<MemoryDatabase>, subroutine: SubroutineEntity) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());

        let result = repo.subroutines_create(subroutine.clone()).await;

        assert!(result.is_ok());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_stores_subroutine(
        db: Arc<MemoryDatabase>,
        subroutine: SubroutineEntity,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());

        repo.subroutines_create(subroutine.clone()).await?;

        let exists = db.subroutines().exists(&subroutine.id())?;
        assert!(exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_fails_when_subroutine_does_not_exist(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let res = repo.subroutines_delete("nonexistent").await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), RepositoryError::NotFound(..)));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_removes_the_record(
        db: Arc<MemoryDatabase>,
        subroutine: SubroutineEntity,
    ) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());
        repo.subroutines_delete(&subroutine.id()).await?;
        let exists = db.subroutines().exists(&subroutine.id())?;
        assert!(!exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_false_for_nonexistent_subroutine(
        db: Arc<MemoryDatabase>,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let query = SubroutinesQuery::default();
        let exists = repo.subroutines_exists(&query).await?;
        assert!(!exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_true_for_existing_subroutine(
        db: Arc<MemoryDatabase>,
        subroutine: SubroutineEntity,
    ) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let query = SubroutinesQuery::builder()
            .for_projector(subroutine.projector_id())
            .for_subroutine_definition(subroutine.subroutine_definition_id())
            .build();
        let exists = repo.subroutines_exists(&query).await?;
        assert!(exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_retrieves_nothing_when_no_records(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());

        let instances = repo.subroutines_find(&SubroutinesQuery::default()).await?;
        assert!(instances.is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_retrieves_nothing_when_no_matches(
        db: Arc<MemoryDatabase>,
        subroutine: SubroutineEntity,
    ) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let query = SubroutinesQuery::builder()
            .for_projector("nonexistent")
            .build();
        let instances = repo.subroutines_find(&query).await?;
        assert!(instances.is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_retrieves_matches(
        db: Arc<MemoryDatabase>,
        subroutine: SubroutineEntity,
    ) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let query = SubroutinesQuery::builder()
            .for_projector(subroutine.projector_id())
            .for_subroutine_definition(subroutine.subroutine_definition_id())
            .build();

        let instances = repo.subroutines_find(&query).await?;
        assert!(!instances.is_empty());
        assert_eq!(instances[0], subroutine);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_fails_when_the_instance_does_not_exist(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());

        let res = repo.subroutines_get("nonexistent").await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), RepositoryError::NotFound(..)));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_retrieves_subroutine(
        db: Arc<MemoryDatabase>,
        subroutine: SubroutineEntity,
    ) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let instance = repo.subroutines_get(&subroutine.id()).await?;
        assert_eq!(instance.id(), subroutine.id());
        Ok(())
    }
}
