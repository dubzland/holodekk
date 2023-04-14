use async_trait::async_trait;

pub use crate::core::{
    entities::Subroutine,
    repositories::{Error, RepositoryQuery, Result, SubroutinesRepository},
};

pub(self) use super::MemoryRepository;

#[async_trait]
impl SubroutinesRepository for MemoryRepository {
    async fn subroutines_create(&self, subroutine: Subroutine) -> Result<Subroutine> {
        self.db.subroutines().add(subroutine.clone())?;
        Ok(subroutine)
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
            .filter(|s| query.matches(s))
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

    use crate::core::{
        entities::subroutine::fixtures::subroutine,
        repositories::{memory::MemoryDatabase, RepositoryId, SubroutinesQuery},
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
        subroutine: Subroutine,
    ) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let res = repo.subroutines_create(subroutine.clone()).await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::AlreadyExists));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_succeeds(db: Arc<MemoryDatabase>, subroutine: Subroutine) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());

        let result = repo.subroutines_create(subroutine.clone()).await;

        assert!(result.is_ok());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_adds_record(db: Arc<MemoryDatabase>, subroutine: Subroutine) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        repo.subroutines_create(subroutine.clone()).await?;

        let new_sub = db.subroutines().get(&subroutine.id())?;
        assert_eq!(new_sub.id(), subroutine.id());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_returns_the_subroutine(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let new_sub = repo.subroutines_create(subroutine.clone()).await?;
        assert_eq!(new_sub.id(), subroutine.id());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_fails_for_nonexistent_subroutine(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let res = repo.subroutines_delete(&subroutine.id()).await;
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
    async fn exists_returns_true_for_existing_subroutine(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
    ) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());
        assert!(repo.subroutines_exists(&subroutine.id()).await?);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_false_for_nonexistent_subroutine(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        assert!(!repo.subroutines_exists(&subroutine.id()).await?);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_nothing_when_no_records(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        assert!(repo
            .subroutines_find(SubroutinesQuery::default())
            .await?
            .is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_nothing_when_no_matches(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
    ) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());
        assert!(repo
            .subroutines_find(SubroutinesQuery::builder().name_eq("nonexistent").build())
            .await?
            .is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_matches(db: Arc<MemoryDatabase>, subroutine: Subroutine) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let res = repo
            .subroutines_find(
                SubroutinesQuery::builder()
                    .name_eq(&subroutine.name)
                    .build(),
            )
            .await?;
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], subroutine);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_fails_when_subroutine_does_not_exist(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let res = repo.subroutines_get(&subroutine.id()).await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotFound));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_returns_subroutine(db: Arc<MemoryDatabase>, subroutine: Subroutine) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let sub = repo.subroutines_get(&subroutine.id()).await?;
        assert_eq!(sub.id(), subroutine.id());
        Ok(())
    }
}
