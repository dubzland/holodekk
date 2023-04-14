use async_trait::async_trait;

pub use crate::core::{
    entities::SubroutineDefinition,
    repositories::{Error, RepositoryQuery, Result, SubroutineDefinitionsRepository},
};

pub(self) use super::MemoryRepository;

#[async_trait]
impl SubroutineDefinitionsRepository for MemoryRepository {
    async fn subroutine_definitions_create(
        &self,
        definition: SubroutineDefinition,
    ) -> Result<SubroutineDefinition> {
        self.db.subroutine_definitions().add(definition.clone())?;
        Ok(definition)
    }

    async fn subroutine_definitions_delete(&self, id: &str) -> Result<()> {
        if self.db.subroutine_definitions().exists(id)? {
            self.db.subroutine_definitions().delete(id)?;
            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }

    async fn subroutine_definitions_exists(&self, id: &str) -> Result<bool> {
        self.db.subroutine_definitions().exists(id)
    }

    async fn subroutine_definitions_find<T>(&self, query: T) -> Result<Vec<SubroutineDefinition>>
    where
        T: RepositoryQuery<Entity = SubroutineDefinition>,
    {
        let definitions = self
            .db
            .subroutine_definitions()
            .all()?
            .into_iter()
            .filter(|s| query.matches(s))
            .collect();
        Ok(definitions)
    }

    async fn subroutine_definitions_get(&self, id: &str) -> Result<SubroutineDefinition> {
        let definition = self.db.subroutine_definitions().get(id)?;
        Ok(definition)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::core::{
        entities::subroutine::definition::fixtures::subroutine_definition,
        repositories::{memory::MemoryDatabase, RepositoryId, SubroutineDefinitionsQuery},
    };

    use super::*;

    #[fixture]
    fn db() -> Arc<MemoryDatabase> {
        Arc::new(MemoryDatabase::new())
    }

    #[rstest]
    #[tokio::test]
    async fn create_fails_when_subroutine_definition_already_exists(
        db: Arc<MemoryDatabase>,
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        db.subroutine_definitions()
            .add(subroutine_definition.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let res = repo
            .subroutine_definitions_create(subroutine_definition.clone())
            .await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::AlreadyExists));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_succeeds(
        db: Arc<MemoryDatabase>,
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());

        let result = repo
            .subroutine_definitions_create(subroutine_definition.clone())
            .await;

        assert!(result.is_ok());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_adds_record(
        db: Arc<MemoryDatabase>,
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        repo.subroutine_definitions_create(subroutine_definition.clone())
            .await?;

        let new_def = db
            .subroutine_definitions()
            .get(&subroutine_definition.id())?;
        assert_eq!(new_def.id(), subroutine_definition.id());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_returns_the_subroutine_definition(
        db: Arc<MemoryDatabase>,
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let new_def = repo
            .subroutine_definitions_create(subroutine_definition.clone())
            .await?;
        assert_eq!(new_def.id(), subroutine_definition.id());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_fails_for_nonexistent_subroutine_definition(
        db: Arc<MemoryDatabase>,
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let res = repo
            .subroutine_definitions_delete(&subroutine_definition.id())
            .await;
        assert!(matches!(res.unwrap_err(), Error::NotFound));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_removes_the_record(
        db: Arc<MemoryDatabase>,
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        db.subroutine_definitions()
            .add(subroutine_definition.clone())?;
        let repo = MemoryRepository::new(db.clone());
        repo.subroutine_definitions_delete(&subroutine_definition.id())
            .await?;

        let exists = db
            .subroutine_definitions()
            .exists(&subroutine_definition.id())?;
        assert!(!exists);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_true_for_existing_subroutine_definition(
        db: Arc<MemoryDatabase>,
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        db.subroutine_definitions()
            .add(subroutine_definition.clone())?;
        let repo = MemoryRepository::new(db.clone());
        assert!(
            repo.subroutine_definitions_exists(&subroutine_definition.id())
                .await?
        );
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn exists_returns_false_for_nonexistent_subroutine_definition(
        db: Arc<MemoryDatabase>,
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        assert!(
            !repo
                .subroutine_definitions_exists(&subroutine_definition.id())
                .await?
        );
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_nothing_when_no_records(db: Arc<MemoryDatabase>) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        assert!(repo
            .subroutine_definitions_find(SubroutineDefinitionsQuery::default())
            .await?
            .is_empty());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_returns_nothing_when_no_matches(
        db: Arc<MemoryDatabase>,
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        db.subroutine_definitions()
            .add(subroutine_definition.clone())?;
        let repo = MemoryRepository::new(db.clone());
        assert!(repo
            .subroutine_definitions_find(
                SubroutineDefinitionsQuery::builder()
                    .name_eq("nonexistent")
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
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        db.subroutine_definitions()
            .add(subroutine_definition.clone())?;
        let repo = MemoryRepository::new(db.clone());
        let res = repo
            .subroutine_definitions_find(
                SubroutineDefinitionsQuery::builder()
                    .name_eq(&subroutine_definition.name)
                    .build(),
            )
            .await?;
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], subroutine_definition);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_fails_when_subroutine_definition_does_not_exist(
        db: Arc<MemoryDatabase>,
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        let res = repo
            .subroutine_definitions_get(&subroutine_definition.id())
            .await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotFound));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_returns_subroutine_definition(
        db: Arc<MemoryDatabase>,
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        db.subroutine_definitions()
            .add(subroutine_definition.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let def = repo
            .subroutine_definitions_get(&subroutine_definition.id())
            .await?;
        assert_eq!(def.id(), subroutine_definition.id());
        Ok(())
    }
}
