use async_trait::async_trait;

pub use crate::core::repositories::{Error, RepositoryQuery, Result};
pub use crate::core::subroutines::{entities::Subroutine, repositories::SubroutinesRepository};

pub(self) use super::MemoryRepository;

#[async_trait]
impl SubroutinesRepository for MemoryRepository {
    async fn subroutines_create(&self, subroutine: Subroutine) -> Result<Subroutine> {
        if self
            .db
            .subroutine_definitions()
            .exists(&subroutine.subroutine_definition_id)?
        {
            self.db.subroutines().add(subroutine.clone())?;
            Ok(subroutine)
        } else {
            Err(Error::RelationNotFound)
        }
    }

    async fn subroutines_delete(&self, id: &str) -> Result<()> {
        if self.db.subroutines().exists(id)? {
            self.db.subroutines().delete(id)?;
            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }

    async fn subroutines_exists(&self, id: &str) -> Result<bool> {
        self.db.subroutines().exists(id)
    }

    async fn subroutines_find<T>(&self, query: T) -> Result<Vec<Subroutine>>
    where
        T: RepositoryQuery<Entity = Subroutine>,
    {
        let subroutines = self
            .db
            .subroutines()
            .all()?
            .into_iter()
            .filter(|i| query.matches(i))
            .collect();
        Ok(subroutines)
    }

    async fn subroutines_get(&self, id: &str) -> Result<Subroutine> {
        let subroutine = self.db.subroutines().get(id)?;
        Ok(subroutine)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::core::repositories::{memory::MemoryDatabase, Error, RepositoryId};
    use crate::core::subroutines::{
        entities::{
            fixtures::{subroutine, subroutine_definition},
            SubroutineDefinition,
        },
        repositories::SubroutinesQuery,
    };

    use super::*;

    #[fixture]
    fn db() -> Arc<MemoryDatabase> {
        Arc::new(MemoryDatabase::new())
    }

    #[rstest]
    #[tokio::test]
    async fn create_fails_when_the_subroutine_definition_does_not_exist(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());

        let result = repo.subroutines_create(subroutine.clone()).await;

        assert!(matches!(result.unwrap_err(), Error::RelationNotFound));

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_fails_when_subroutine_already_exists(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        db.subroutine_definitions().add(subroutine_definition)?;
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let result = repo.subroutines_create(subroutine.clone()).await;

        assert!(matches!(result.unwrap_err(), Error::AlreadyExists));

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_succeeds(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        db.subroutine_definitions().add(subroutine_definition)?;
        let repo = MemoryRepository::new(db.clone());

        let result = repo.subroutines_create(subroutine.clone()).await;

        assert!(result.is_ok());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_stores_subroutine(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        db.subroutine_definitions().add(subroutine_definition)?;
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
        assert!(matches!(res.unwrap_err(), Error::NotFound));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_removes_the_record(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
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
        let exists = repo.subroutines_exists("nonexistent").await?;
        assert!(!exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_true_for_existing_subroutine(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
    ) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let exists = repo.subroutines_exists(&subroutine.id()).await?;
        assert!(exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_retrieves_nothing_when_no_records(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());

        let instances = repo.subroutines_find(SubroutinesQuery::default()).await?;
        assert!(instances.is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_retrieves_nothing_when_no_matches(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
    ) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let instances = repo
            .subroutines_find(SubroutinesQuery::builder().fleet_eq("nonexistent").build())
            .await?;
        assert!(instances.is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_retrieves_matches(db: Arc<MemoryDatabase>, subroutine: Subroutine) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let instances = repo
            .subroutines_find(
                SubroutinesQuery::builder()
                    .fleet_eq(&subroutine.fleet)
                    .namespace_eq(&subroutine.namespace)
                    .build(),
            )
            .await?;
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
        assert!(matches!(res.unwrap_err(), Error::NotFound));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_retrieves_subroutine(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
    ) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let instance = repo.subroutines_get(&subroutine.id()).await?;
        assert_eq!(instance.id(), subroutine.id());
        Ok(())
    }
}
