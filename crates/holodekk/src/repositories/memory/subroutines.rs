use async_trait::async_trait;

pub(self) use crate::entities::Subroutine;
pub(self) use crate::repositories::{Result, SubroutineRepository};

pub(self) use super::MemoryRepository;

#[async_trait]
impl SubroutineRepository for MemoryRepository {
    async fn subroutine_create(&self, subroutine: Subroutine) -> Result<Subroutine> {
        self.db.subroutines().add(subroutine.clone())?;
        Ok(subroutine)
    }

    async fn subroutine_get(&self, id: &str, include_instances: bool) -> Result<Subroutine> {
        let mut subroutine = self.db.subroutines().get(id)?;
        if include_instances {
            let instances = self
                .db
                .subroutine_instances()
                .get_all_by_subroutine(&subroutine)?;
            subroutine.instances = Some(instances);
        }
        Ok(subroutine)
    }

    async fn subroutine_get_by_name(
        &self,
        name: &str,
        include_instances: bool,
    ) -> Result<Subroutine> {
        let mut subroutine = self.db.subroutines().get_by_name(name)?;
        if include_instances {
            let instances = self
                .db
                .subroutine_instances()
                .get_all_by_subroutine(&subroutine)?;
            subroutine.instances = Some(instances);
        }
        Ok(subroutine)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::entities::subroutine::fixtures::subroutine;
    use crate::repositories::memory::{MemoryDatabase, MemoryDatabaseKey};

    use super::*;

    #[fixture]
    fn db() -> Arc<MemoryDatabase> {
        Arc::new(MemoryDatabase::new())
    }

    #[rstest]
    #[tokio::test]
    async fn creates_subroutine(db: Arc<MemoryDatabase>, subroutine: Subroutine) -> Result<()> {
        let repo = MemoryRepository::new(db.clone());

        let result = repo.subroutine_create(subroutine.clone()).await;

        assert!(result.is_ok());

        let new_sub = db.subroutines().get(&subroutine.db_key())?;
        assert_eq!(new_sub.id, subroutine.id);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn retrieves_subroutine(db: Arc<MemoryDatabase>, subroutine: Subroutine) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let sub = repo.subroutine_get(&subroutine.id, false).await?;
        assert_eq!(sub.id, subroutine.id);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn retrieves_subroutine_by_name(
        db: Arc<MemoryDatabase>,
        subroutine: Subroutine,
    ) -> Result<()> {
        db.subroutines().add(subroutine.clone())?;
        let repo = MemoryRepository::new(db.clone());

        let sub = repo.subroutine_get_by_name(&subroutine.name, false).await?;
        assert_eq!(sub.id, subroutine.id);
        Ok(())
    }
}
