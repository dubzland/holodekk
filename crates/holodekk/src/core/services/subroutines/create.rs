use std::path::PathBuf;

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::core::{
    entities::{Subroutine, SubroutineKind},
    repositories::{subroutine_repo_id, SubroutinesRepository},
    services::{Error, Result},
};

use super::SubroutinesService;

#[derive(Clone, Debug)]
pub struct SubroutinesCreateInput {
    pub name: String,
    pub path: PathBuf,
    pub kind: SubroutineKind,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Create {
    async fn create(&self, input: SubroutinesCreateInput) -> Result<Subroutine>;
}

#[async_trait]
impl<T> Create for SubroutinesService<T>
where
    T: SubroutinesRepository,
{
    /// Creates a Subroutine entry in the repository.
    async fn create(&self, input: SubroutinesCreateInput) -> Result<Subroutine> {
        // make sure this subroutine does not already exist
        println!("Checking for subroutine with name: {}", input.name,);
        if self
            .repo
            .subroutines_get(&subroutine_repo_id(&input.name))
            .await
            .is_ok()
        {
            return Err(Error::Duplicate);
        }

        let subroutine = Subroutine::new(input.name, input.path, input.kind);
        let subroutine = self.repo.subroutines_create(subroutine).await?;
        Ok(subroutine)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::core::{
        entities::{subroutine::fixtures::subroutine, Subroutine},
        repositories::{self, fixtures::subroutines_repository, MockSubroutinesRepository},
        services::Error,
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn creates_subroutine(
        mut subroutines_repository: MockSubroutinesRepository,
        subroutine: Subroutine,
    ) -> Result<()> {
        let input = SubroutinesCreateInput {
            name: subroutine.name.clone(),
            path: subroutine.path.clone(),
            kind: subroutine.kind,
        };

        let sub_name = subroutine.name.clone();
        subroutines_repository
            .expect_subroutines_get()
            .withf(move |name| name == &subroutine_repo_id(&sub_name))
            .return_const(Err(repositories::Error::NotFound));

        let sub_path = subroutine.path.clone();
        let sub_name = subroutine.name.clone();

        subroutines_repository
            .expect_subroutines_create()
            .withf(move |new_sub: &Subroutine| {
                (*new_sub).path.eq(&sub_path) && (*new_sub).name.eq(&sub_name)
            })
            .return_const(Ok(subroutine.clone()));

        let service = SubroutinesService::new(Arc::new(subroutines_repository));

        let sub = service.create(input).await?;
        assert_eq!(&sub, &subroutine);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn rejects_duplicate_subroutine_name(
        mut subroutines_repository: MockSubroutinesRepository,
        subroutine: Subroutine,
    ) {
        let input = SubroutinesCreateInput {
            name: subroutine.name.clone(),
            path: subroutine.path.clone(),
            kind: subroutine.kind,
        };

        let sub_name = subroutine.name.clone();

        subroutines_repository
            .expect_subroutines_get()
            .withf(move |name| name == &subroutine_repo_id(&sub_name))
            .return_const(Ok(subroutine.to_owned()));

        let service = SubroutinesService::new(Arc::new(subroutines_repository));

        let res = service.create(input).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::Duplicate);
    }
}
