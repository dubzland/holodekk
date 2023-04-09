use std::path::PathBuf;

use crate::entities::{Subroutine, SubroutineKind};
use crate::repositories::Repository;
use crate::services::{Error, Result};

use super::SubroutinesService;

#[derive(Clone, Debug)]
pub struct SubroutineCreateInput<S, P>
where
    S: Into<String> + AsRef<str> + std::fmt::Display,
    P: Into<PathBuf>,
{
    pub name: S,
    pub path: P,
    pub kind: SubroutineKind,
}

impl<T> SubroutinesService<T>
where
    T: Repository,
{
    /// Creates a Subroutine entry in the repository.
    pub async fn create<S, P>(&self, input: SubroutineCreateInput<S, P>) -> Result<Subroutine>
    where
        S: Into<String> + AsRef<str> + std::fmt::Display,
        P: Into<PathBuf>,
    {
        // make sure this subroutine does not already exist
        println!("Checking for subroutine with name: {}", input.name,);
        if self
            .repo
            .subroutine_get_by_name(input.name.as_ref(), false)
            .await
            .is_ok()
        {
            return Err(Error::Duplicate);
        }

        let subroutine = Subroutine::new(input.name, input.path, input.kind);
        let subroutine = self.repo.subroutine_create(subroutine).await?;
        Ok(subroutine)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::entities::fixtures::subroutine;
    use crate::entities::Subroutine;
    use crate::repositories::fixtures::repository;
    use crate::repositories::MockRepository;
    use crate::services::Error;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn creates_subroutine(
        mut repository: MockRepository,
        subroutine: &Subroutine,
    ) -> Result<()> {
        let input = SubroutineCreateInput {
            name: &subroutine.name.clone(),
            path: subroutine.path.clone(),
            kind: subroutine.kind,
        };

        let sub_name = subroutine.name.clone();
        repository
            .expect_subroutine_get_by_name()
            .withf(move |name, _inc| name == &sub_name)
            .return_const(Err(crate::repositories::Error::NotFound));

        let sub_path = subroutine.path.clone();
        let sub_name = subroutine.name.clone();

        repository
            .expect_subroutine_create()
            .withf(move |new_sub: &Subroutine| {
                (*new_sub).path.eq(&sub_path) && (*new_sub).name.eq(&sub_name)
            })
            .return_const(Ok(subroutine.clone()));

        let service = SubroutinesService::new(Arc::new(repository), "test-fleet", "test-namespace");

        let sub = service.create(input).await?;
        assert_eq!(&sub, subroutine);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn rejects_duplicate_subroutine_name(
        mut repository: MockRepository,
        subroutine: &Subroutine,
    ) {
        let input = SubroutineCreateInput {
            name: &subroutine.name,
            path: subroutine.path.clone(),
            kind: subroutine.kind,
        };

        let sub_name = subroutine.name.clone();

        repository
            .expect_subroutine_get_by_name()
            .withf(move |name, _inc| name == &sub_name)
            .return_const(Ok(subroutine.to_owned()));

        let service = SubroutinesService::new(Arc::new(repository), "test-fleet", "test-namespace");

        let res = service.create(input).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::Duplicate);
    }
}
