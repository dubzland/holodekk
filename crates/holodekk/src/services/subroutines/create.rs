use std::path::PathBuf;

use crate::entities::{Subroutine, SubroutineStatus};
use crate::repositories::Repository;
use crate::services::{Error, Result};

use super::SubroutinesService;

#[derive(Clone, Debug)]
pub struct SubroutineCreateInput {
    pub name: String,
    pub path: PathBuf,
}

impl<T> SubroutinesService<T>
where
    T: Repository,
{
    /// Creates a Subroutine entry in the repository.
    pub async fn create(&self, input: SubroutineCreateInput) -> Result<Subroutine> {
        // make sure this subroutine does not already exist
        println!("Checking for subroutine with name: {}", input.name);
        if self
            .repo
            .subroutine_get(&self.fleet, &self.namespace, &input.name)
            .await
            .is_ok()
        {
            return Err(Error::Duplicate);
        }

        let subroutine = Subroutine {
            fleet: self.fleet.clone(),
            namespace: self.namespace.clone(),
            name: input.name.to_owned(),
            path: input.path.to_owned(),
            status: SubroutineStatus::Stopped,
        };
        let subroutine = self.repo.subroutine_create(subroutine).await?;
        Ok(subroutine)
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
    async fn creates_subroutine(
        mut repository: MockRepository,
        subroutine: Subroutine,
    ) -> Result<()> {
        let input = SubroutineCreateInput {
            name: subroutine.name.clone(),
            path: subroutine.path.clone(),
        };

        let sub_name = subroutine.name.clone();
        repository
            .expect_subroutine_get()
            .withf(move |fleet, namespace, name| {
                fleet == "test-fleet" && namespace == "test-namespace" && name == &sub_name
            })
            .return_const(Err(crate::repositories::Error::NotFound));

        repository
            .expect_subroutine_create()
            .withf(|new_sub: &Subroutine| {
                (*new_sub).name.eq("test") && (*new_sub).path.eq(&PathBuf::from("/tmp"))
            })
            .return_const(Ok(subroutine.clone()));

        let service = SubroutinesService::new(Arc::new(repository), "test-fleet", "test-namespace");

        let sub = service.create(input).await?;
        assert_eq!(sub, subroutine);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn rejects_duplicate_subroutine_name(
        mut repository: MockRepository,
        subroutine: Subroutine,
    ) {
        let input = SubroutineCreateInput {
            name: "test".into(),
            path: "/tmp".into(),
        };

        repository
            .expect_subroutine_get()
            .withf(|fleet, namespace, name| {
                fleet == "test-fleet" && namespace == "test-namespace" && name == "test"
            })
            .return_const(Ok(subroutine.clone()));

        let service = SubroutinesService::new(Arc::new(repository), "test-fleet", "test-namespace");

        let res = service.create(input).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::Duplicate);
    }
}
