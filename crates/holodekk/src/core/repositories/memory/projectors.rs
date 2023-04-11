use async_trait::async_trait;

use crate::core::entities::Projector;
use crate::core::repositories::{Error, ProjectorRepository, Result};

use super::{MemoryDatabaseKey, MemoryRepository};

#[async_trait]
impl ProjectorRepository for MemoryRepository {
    async fn projector_create(&self, projector: Projector) -> Result<Projector> {
        // Ensure the projector doesn't exist
        if self.db.projectors().exists(&projector.db_key()) {
            Err(Error::AlreadyExists)
        } else {
            self.db.projectors().add(projector.clone())?;
            Ok(projector)
        }
    }

    async fn projector_get(&self, id: &str) -> Result<Projector> {
        self.db.projectors().get(id)
    }

    async fn projector_delete(&self, id: &str) -> Result<()> {
        self.db.projectors().delete(id)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::core::entities::projector::fixtures::projector;
    use crate::core::repositories::{
        self,
        memory::{MemoryDatabase, MemoryDatabaseKey},
    };

    use super::*;

    #[fixture]
    fn db() -> Arc<MemoryDatabase> {
        Arc::new(MemoryDatabase::new())
    }

    #[rstest]
    #[tokio::test]
    async fn creates_projector(db: Arc<MemoryDatabase>, projector: Projector) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());

        let result = repo.projector_create(projector.clone()).await;

        assert!(result.is_ok());

        let new_projector = db.projectors().get(&projector.db_key())?;
        assert_eq!(new_projector.id, projector.id);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn prevents_creating_duplicates(
        db: Arc<MemoryDatabase>,
        projector: Projector,
    ) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());
        db.projectors().add(projector.clone())?;

        let result = repo.projector_create(projector.clone()).await;

        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, repositories::Error::AlreadyExists));
        Ok(())
    }
}
