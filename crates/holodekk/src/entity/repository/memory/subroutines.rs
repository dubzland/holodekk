use async_trait::async_trait;
use timestamps::Timestamps;

use crate::entity::repository::{Error, Query as RepositoryQuery, Result};
pub use crate::images::SubroutineImageId;
use crate::subroutine::{
    entity::{repository::Query, Id, Repository, Status},
    Entity,
};

use super::Memory;

#[async_trait]
impl Repository for Memory {
    async fn subroutines_create(&self, mut subroutine: Entity) -> Result<Entity> {
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

    async fn subroutines_delete(&self, id: &Id) -> Result<()> {
        if self.db.subroutines().exists(id)? {
            self.db.subroutines().delete(id)?;
            Ok(())
        } else {
            Err(Error::NotFound(id.to_string().try_into().unwrap()))
        }
    }

    async fn subroutines_exists<'a>(&self, query: Query<'a>) -> Result<bool> {
        Ok(self.db.subroutines().all().iter().any(|s| query.matches(s)))
    }

    async fn subroutines_find<'a>(&self, query: Query<'a>) -> Result<Vec<Entity>> {
        Ok(self
            .db
            .subroutines()
            .all()
            .into_iter()
            .filter(|i| query.matches(i))
            .collect())
    }

    async fn subroutines_get(&self, id: &Id) -> Result<Entity> {
        let subroutine = self.db.subroutines().get(id)?;
        Ok(subroutine)
    }

    async fn subroutines_update(&self, id: &Id, status: Option<Status>) -> Result<Entity> {
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

    use crate::entity::repository::memory::Database;
    use crate::scene;
    use crate::subroutine::entity::mock_entity;

    use super::*;

    #[fixture]
    fn db() -> Arc<Database> {
        Arc::new(Database::new())
    }

    #[rstest]
    #[tokio::test]
    async fn create_fails_when_subroutine_already_exists(
        db: Arc<Database>,
        mock_entity: Entity,
    ) -> Result<()> {
        db.subroutines().add(mock_entity.clone())?;
        let repo = Memory::new(db.clone());

        let result = repo.subroutines_create(mock_entity).await;

        assert!(matches!(result.unwrap_err(), Error::Conflict(..)));

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_succeeds(db: Arc<Database>, mock_entity: Entity) -> Result<()> {
        let repo = Memory::new(db.clone());

        let result = repo.subroutines_create(mock_entity).await;

        assert!(result.is_ok());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_stores_subroutine(db: Arc<Database>, mock_entity: Entity) -> Result<()> {
        let repo = Memory::new(db.clone());

        repo.subroutines_create(mock_entity.clone()).await?;

        let exists = db.subroutines().exists(&mock_entity.id)?;
        assert!(exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_fails_when_subroutine_does_not_exist(db: Arc<Database>) -> Result<()> {
        let repo = Memory::new(db.clone());
        let res = repo.subroutines_delete(&Id::generate()).await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotFound(..)));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_removes_the_record(db: Arc<Database>, mock_entity: Entity) -> Result<()> {
        db.subroutines().add(mock_entity.clone())?;
        let repo = Memory::new(db.clone());
        repo.subroutines_delete(&mock_entity.id).await?;
        let exists = db.subroutines().exists(&mock_entity.id)?;
        assert!(!exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_false_for_nonexistent_subroutine(db: Arc<Database>) -> Result<()> {
        let repo = Memory::new(db.clone());
        let query = Query::default();
        let exists = repo.subroutines_exists(query).await?;
        assert!(!exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_true_for_existing_subroutine(
        db: Arc<Database>,
        mock_entity: Entity,
    ) -> Result<()> {
        db.subroutines().add(mock_entity.clone())?;
        let repo = Memory::new(db.clone());
        let query = Query::builder()
            .for_scene_entity(&mock_entity.scene_entity_id)
            .for_subroutine_image(&mock_entity.subroutine_image_id)
            .build();
        let exists = repo.subroutines_exists(query).await?;
        assert!(exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_retrieves_nothing_when_no_records(db: Arc<Database>) -> Result<()> {
        let repo = Memory::new(db.clone());

        let instances = repo.subroutines_find(Query::default()).await?;
        assert!(instances.is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_retrieves_nothing_when_no_matches(
        db: Arc<Database>,
        mock_entity: Entity,
    ) -> Result<()> {
        db.subroutines().add(mock_entity.clone())?;
        let repo = Memory::new(db.clone());
        let invalid_id = scene::entity::Id::generate();

        let query = Query::builder().for_scene_entity(&invalid_id).build();
        let instances = repo.subroutines_find(query).await?;
        assert!(instances.is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_retrieves_matches(db: Arc<Database>, mock_entity: Entity) -> Result<()> {
        db.subroutines().add(mock_entity.clone())?;
        let repo = Memory::new(db.clone());
        let query = Query::builder()
            .for_scene_entity(&mock_entity.scene_entity_id)
            .for_subroutine_image(&mock_entity.subroutine_image_id)
            .build();

        let instances = repo.subroutines_find(query).await?;
        assert!(!instances.is_empty());
        assert_eq!(instances[0], mock_entity);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_fails_when_the_instance_does_not_exist(db: Arc<Database>) -> Result<()> {
        let repo = Memory::new(db.clone());
        let invalid_id = Id::generate();

        let res = repo.subroutines_get(&invalid_id).await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotFound(..)));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_retrieves_subroutine(db: Arc<Database>, mock_entity: Entity) -> Result<()> {
        db.subroutines().add(mock_entity.clone())?;
        let repo = Memory::new(db.clone());

        let instance = repo.subroutines_get(&mock_entity.id).await?;
        assert_eq!(instance.id, mock_entity.id);
        Ok(())
    }
}
