use crate::entities::SubroutineStatus;
use crate::repositories::Repository;
use crate::services::{Error, Result};

use super::SubroutinesService;

impl<T> SubroutinesService<T>
where
    T: Repository,
{
    pub async fn status(&self, name: &str) -> Result<SubroutineStatus> {
        let subroutine = self
            .repo
            .subroutine_get(&self.fleet, &self.namespace, name)
            .await
            .map_err(|e| match e {
                crate::repositories::Error::NotFound => Error::NotFound,
                _ => Error::Repository(e),
            })?;

        Ok(subroutine.status)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use rstest::*;

    use crate::entities::{Subroutine, SubroutineStatus};
    use crate::repositories::MockRepository;
    use crate::services::Error;

    use super::*;

    #[fixture]
    fn repository() -> MockRepository {
        MockRepository::default()
    }

    #[fixture]
    fn subroutine() -> Subroutine {
        Subroutine {
            fleet: "test-fleet".to_string(),
            namespace: "test-namespace".to_string(),
            name: "test".to_string(),
            path: PathBuf::from("/tmp"),
            status: SubroutineStatus::Stopped,
        }
    }

    #[rstest]
    #[tokio::test]
    async fn returns_status_for_existing_subroutine(
        mut repository: MockRepository,
        subroutine: Subroutine,
    ) -> Result<()> {
        let test_fleet = "test-fleet";
        let test_namespace = "test-namespace";
        let sub_name = subroutine.name.clone();

        repository
            .expect_subroutine_get()
            .withf(move |fleet, namespace, name| {
                fleet == test_fleet && namespace == test_namespace && name == &sub_name
            })
            .return_const(Ok(subroutine.clone()));

        let service = SubroutinesService::new(
            Arc::new(repository),
            test_fleet.clone(),
            test_namespace.clone(),
        );

        let status = service.status(&subroutine.name).await?;
        assert_eq!(status, SubroutineStatus::Stopped);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn returns_not_found_for_missing_subroutine(
        mut repository: MockRepository,
    ) -> Result<()> {
        let test_fleet = "test-fleet";
        let test_namespace = "test-namespace";
        let sub_name = "test/sub";

        repository
            .expect_subroutine_get()
            .withf(move |fleet, namespace, name| {
                fleet == test_fleet && namespace == test_namespace && name == sub_name
            })
            .return_const(Err(crate::repositories::Error::NotFound));

        let service = SubroutinesService::new(
            Arc::new(repository),
            test_fleet.clone(),
            test_namespace.clone(),
        );

        let res = service.status(&sub_name).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::NotFound);
        Ok(())
    }
}
