use async_trait::async_trait;

use crate::entities::Subroutine;
use crate::repositories::{Repository, Result};

use super::{subroutine_key, MemoryRepository};

#[async_trait]
impl Repository for MemoryRepository {
    async fn subroutine_create(&self, subroutine: Subroutine) -> Result<Subroutine> {
        self.db.subroutines().add(subroutine.clone())?;
        Ok(subroutine)
    }

    async fn subroutine_get<'a>(
        &self,
        fleet: &'a str,
        namespace: &'a str,
        name: &'a str,
    ) -> Result<Subroutine> {
        let key = subroutine_key(fleet, namespace, name);
        self.db.subroutines().get(&key)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::repositories::memory::{MemoryDatabase, MemoryDatabaseKey};

    use super::*;

    #[fixture]
    fn db() -> Arc<MemoryDatabase> {
        Arc::new(MemoryDatabase::new())
    }

    #[fixture]
    fn subroutine() -> Subroutine {
        Subroutine::new("test-fleet", "test-namespace", "test/sub", "/tmp")
    }

    #[rstest]
    #[tokio::test]
    async fn creates_subroutine(db: Arc<MemoryDatabase>, subroutine: Subroutine) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());

        let result = repo.subroutine_create(subroutine.clone()).await;

        assert!(result.is_ok());

        let new_sub = db.subroutines().get(&subroutine.db_key())?;
        assert_eq!(new_sub.name, subroutine.name);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn retrieves_subroutine(db: Arc<MemoryDatabase>, subroutine: Subroutine) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let sub = repo
            .subroutine_get(&subroutine.fleet, &subroutine.namespace, &subroutine.name)
            .await?;
        assert_eq!(sub.name, subroutine.name);
        Ok(())
    }
}
