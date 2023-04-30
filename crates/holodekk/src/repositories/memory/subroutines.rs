use async_trait::async_trait;
use timestamps::Timestamps;

pub use crate::core::{
    entities::{SceneEntityId, SubroutineDefinitionEntityId, SubroutineEntity, SubroutineEntityId},
    enums::SubroutineStatus,
    repositories::{Error, RepositoryQuery, Result, SubroutinesQuery, SubroutinesRepository},
};

pub(self) use super::MemoryRepository;

#[async_trait]
impl SubroutinesRepository for MemoryRepository {
    async fn subroutines_create(
        &self,
        mut subroutine: SubroutineEntity,
    ) -> Result<SubroutineEntity> {
        match self.subroutines_get(&subroutine.id).await {
            Err(Error::NotFound(_)) => {
                subroutine.created();
                subroutine.updated();
                self.db.subroutines().add(subroutine.clone())?;
                Ok(subroutine)
            }
            Ok(_) => Err(Error::Conflict(format!(
                "Subroutine already exists with id {}",
                subroutine.id
            ))),
            Err(err) => Err(err),
        }
    }

    async fn subroutines_delete(&self, id: &SubroutineEntityId) -> Result<()> {
        if self.db.subroutines().exists(id)? {
            self.db.subroutines().delete(id)?;
            Ok(())
        } else {
            Err(Error::NotFound(id.to_string().try_into().unwrap()))
        }
    }

    async fn subroutines_exists<'a>(&self, query: SubroutinesQuery<'a>) -> Result<bool> {
        Ok(self.db.subroutines().all().iter().any(|s| query.matches(s)))
    }

    async fn subroutines_find<'a>(
        &self,
        query: SubroutinesQuery<'a>,
    ) -> Result<Vec<SubroutineEntity>> {
        Ok(self
            .db
            .subroutines()
            .all()
            .into_iter()
            .filter(|i| query.matches(i))
            .collect())
    }

    async fn subroutines_get(&self, id: &SubroutineEntityId) -> Result<SubroutineEntity> {
        let subroutine = self.db.subroutines().get(id)?;
        Ok(subroutine)
    }

    async fn subroutines_update(
        &self,
        id: &SubroutineEntityId,
        status: Option<SubroutineStatus>,
    ) -> Result<SubroutineEntity> {
        let mut subroutine = self.subroutines_get(id).await?;
        if let Some(status) = status {
            subroutine.status = status;
        }
        subroutine.updated();
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::{
        core::{
            entities::fixtures::mock_subroutine,
            repositories::{Error, SubroutinesQuery},
        },
        repositories::memory::MemoryDatabase,
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
        mock_subroutine: SubroutineEntity,
    ) -> Result<()> {
        db.subroutines().add(mock_subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let result = repo.subroutines_create(mock_subroutine).await;

        assert!(matches!(result.unwrap_err(), Error::Conflict(..)));

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_succeeds(
        db: Arc<MemoryDatabase>,
        mock_subroutine: SubroutineEntity,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());

        let result = repo.subroutines_create(mock_subroutine).await;

        assert!(result.is_ok());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_stores_subroutine(
        db: Arc<MemoryDatabase>,
        mock_subroutine: SubroutineEntity,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());

        repo.subroutines_create(mock_subroutine.clone()).await?;

        let exists = db.subroutines().exists(&mock_subroutine.id)?;
        assert!(exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_fails_when_subroutine_does_not_exist(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let res = repo
            .subroutines_delete(&SubroutineEntityId::generate())
            .await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotFound(..)));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_removes_the_record(
        db: Arc<MemoryDatabase>,
        mock_subroutine: SubroutineEntity,
    ) -> Result<()> {
        db.subroutines().add(mock_subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());
        repo.subroutines_delete(&mock_subroutine.id).await?;
        let exists = db.subroutines().exists(&mock_subroutine.id)?;
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
        let exists = repo.subroutines_exists(query).await?;
        assert!(!exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_true_for_existing_subroutine(
        db: Arc<MemoryDatabase>,
        mock_subroutine: SubroutineEntity,
    ) -> Result<()> {
        db.subroutines().add(mock_subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let query = SubroutinesQuery::builder()
            .for_scene(&mock_subroutine.scene_id)
            .for_subroutine_definition(&mock_subroutine.subroutine_definition_id)
            .build();
        let exists = repo.subroutines_exists(query).await?;
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
        mock_subroutine: SubroutineEntity,
    ) -> Result<()> {
        db.subroutines().add(mock_subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let invalid_id = SceneEntityId::generate();

        let query = SubroutinesQuery::builder().for_scene(&invalid_id).build();
        let instances = repo.subroutines_find(query).await?;
        assert!(instances.is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_retrieves_matches(
        db: Arc<MemoryDatabase>,
        mock_subroutine: SubroutineEntity,
    ) -> Result<()> {
        db.subroutines().add(mock_subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let query = SubroutinesQuery::builder()
            .for_scene(&mock_subroutine.scene_id)
            .for_subroutine_definition(&mock_subroutine.subroutine_definition_id)
            .build();

        let instances = repo.subroutines_find(query).await?;
        assert!(!instances.is_empty());
        assert_eq!(instances[0], mock_subroutine);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_fails_when_the_instance_does_not_exist(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let invalid_id = SubroutineEntityId::generate();

        let res = repo.subroutines_get(&invalid_id).await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotFound(..)));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_retrieves_subroutine(
        db: Arc<MemoryDatabase>,
        mock_subroutine: SubroutineEntity,
    ) -> Result<()> {
        db.subroutines().add(mock_subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let instance = repo.subroutines_get(&mock_subroutine.id).await?;
        assert_eq!(instance.id, mock_subroutine.id);
        Ok(())
    }
}
