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
            .subroutine_get_by_name(name)
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
    use std::{path::PathBuf, sync::Arc};

    use mockall::predicate::*;
    use rstest::*;

    use super::*;
    use crate::entities::Subroutine;
    use crate::repositories::MockRepository;
    use crate::services::Error;

    #[fixture]
    fn repository() -> MockRepository {
        MockRepository::new()
    }

    #[fixture]
    fn subroutine() -> Subroutine {
        Subroutine {
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
    ) {
        repository
            .expect_subroutine_get_by_name()
            .with(eq("test"))
            .returning(move |_| Ok(subroutine.clone()));

        let service = SubroutinesService::new(Arc::new(repository));

        let res = service.status("test").await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), SubroutineStatus::Stopped);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_not_found_for_missing_subroutine(mut repository: MockRepository) {
        repository
            .expect_subroutine_get_by_name()
            .with(eq("test"))
            .returning(move |_| Err(crate::repositories::Error::NotFound));

        let service = SubroutinesService::new(Arc::new(repository));

        let res = service.status("test").await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::NotFound);
    }
}
