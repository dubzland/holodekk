use async_trait::async_trait;

pub use crate::core::entities::SubroutineInstance;
pub use crate::core::repositories::{
    Error, RepositoryQuery, Result, SubroutineInstancesRepository,
};

pub(self) use super::MemoryRepository;

#[async_trait]
impl SubroutineInstancesRepository for MemoryRepository {
    async fn subroutine_instances_create(
        &self,
        instance: SubroutineInstance,
    ) -> Result<SubroutineInstance> {
        if self.db.subroutines().exists(&instance.subroutine_id)? {
            self.db.subroutine_instances().add(instance.clone())?;
            Ok(instance)
        } else {
            Err(Error::RelationNotFound)
        }
    }

    async fn subroutine_instances_delete(&self, id: &str) -> Result<()> {
        if self.db.subroutine_instances().exists(id)? {
            self.db.subroutine_instances().delete(id)?;
            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }

    async fn subroutine_instances_exists(&self, id: &str) -> Result<bool> {
        self.db.subroutine_instances().exists(id)
    }

    async fn subroutine_instances_find<T>(&self, query: T) -> Result<Vec<SubroutineInstance>>
    where
        T: RepositoryQuery<Entity = SubroutineInstance>,
    {
        let instances = self
            .db
            .subroutine_instances()
            .all()?
            .into_iter()
            .filter(|i| query.matches(i))
            .collect();
        Ok(instances)
    }

    async fn subroutine_instances_get(&self, id: &str) -> Result<SubroutineInstance> {
        let instance = self.db.subroutine_instances().get(id)?;
        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::core::{
        entities::subroutine::{
            fixtures::subroutine, instance::fixtures::subroutine_instance, Subroutine,
        },
        repositories::{memory::MemoryDatabase, Error, RepositoryId, SubroutineInstancesQuery},
    };

    use super::*;

    #[fixture]
    fn db() -> Arc<MemoryDatabase> {
        Arc::new(MemoryDatabase::new())
    }

    #[rstest]
    #[tokio::test]
    async fn create_fails_when_the_subroutine_does_not_exist(
        db: Arc<MemoryDatabase>,
        subroutine_instance: SubroutineInstance,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());

        let result = repo
            .subroutine_instances_create(subroutine_instance.clone())
            .await;

        assert!(matches!(result.unwrap_err(), Error::RelationNotFound));

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_fails_when_instance_already_exists(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
        subroutine_instance: SubroutineInstance,
    ) -> Result<()> {
        db.subroutines().add(subroutine)?;
        db.subroutine_instances().add(subroutine_instance.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let result = repo
            .subroutine_instances_create(subroutine_instance.clone())
            .await;

        assert!(matches!(result.unwrap_err(), Error::AlreadyExists));

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_succeeds(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
        subroutine_instance: SubroutineInstance,
    ) -> Result<()> {
        db.subroutines().add(subroutine)?;
        let repo = MemoryRepository::new(db.clone());

        let result = repo
            .subroutine_instances_create(subroutine_instance.clone())
            .await;

        assert!(result.is_ok());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_stores_subroutine_instance(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
        subroutine_instance: SubroutineInstance,
    ) -> Result<()> {
        db.subroutines().add(subroutine)?;
        let repo = MemoryRepository::new(db.clone());

        repo.subroutine_instances_create(subroutine_instance.clone())
            .await?;

        let exists = db
            .subroutine_instances()
            .exists(&subroutine_instance.id())?;
        assert!(exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_fails_when_instance_does_not_exist(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let res = repo.subroutine_instances_delete("nonexistent").await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotFound));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_removes_the_record(
        db: Arc<MemoryDatabase>,
        subroutine_instance: SubroutineInstance,
    ) -> Result<()> {
        db.subroutine_instances().add(subroutine_instance.clone())?;
        let repo = MemoryRepository::new(db.clone());
        repo.subroutine_instances_delete(&subroutine_instance.id())
            .await?;
        let exists = db
            .subroutine_instances()
            .exists(&subroutine_instance.id())?;
        assert!(!exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_false_for_nonexistent_instance(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let exists = repo.subroutine_instances_exists("nonexistent").await?;
        assert!(!exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_true_for_existing_instance(
        db: Arc<MemoryDatabase>,
        subroutine_instance: SubroutineInstance,
    ) -> Result<()> {
        db.subroutine_instances().add(subroutine_instance.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let exists = repo
            .subroutine_instances_exists(&subroutine_instance.id())
            .await?;
        assert!(exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_retrieves_nothing_when_no_records(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());

        let instances = repo
            .subroutine_instances_find(SubroutineInstancesQuery::default())
            .await?;
        assert!(instances.is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_retrieves_nothing_when_no_matches(
        db: Arc<MemoryDatabase>,
        subroutine_instance: SubroutineInstance,
    ) -> Result<()> {
        db.subroutine_instances().add(subroutine_instance.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let instances = repo
            .subroutine_instances_find(
                SubroutineInstancesQuery::builder()
                    .fleet_eq("nonexistent")
                    .build(),
            )
            .await?;
        assert!(instances.is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_retrieves_matches(
        db: Arc<MemoryDatabase>,
        subroutine_instance: SubroutineInstance,
    ) -> Result<()> {
        db.subroutine_instances().add(subroutine_instance.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let instances = repo
            .subroutine_instances_find(
                SubroutineInstancesQuery::builder()
                    .fleet_eq(&subroutine_instance.fleet)
                    .namespace_eq(&subroutine_instance.namespace)
                    .build(),
            )
            .await?;
        assert!(!instances.is_empty());
        assert_eq!(instances[0], subroutine_instance);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_fails_when_the_instance_does_not_exist(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());

        let res = repo.subroutine_instances_get("nonexistent").await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotFound));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_retrieves_subroutine_instance(
        db: Arc<MemoryDatabase>,
        subroutine_instance: SubroutineInstance,
    ) -> Result<()> {
        db.subroutine_instances().add(subroutine_instance.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let instance = repo
            .subroutine_instances_get(&subroutine_instance.id())
            .await?;
        assert_eq!(instance.id(), subroutine_instance.id());
        Ok(())
    }
}
