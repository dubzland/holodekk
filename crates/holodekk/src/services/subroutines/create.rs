use std::path::PathBuf;

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::entities::{Subroutine, SubroutineKind};
use crate::repositories::SubroutineRepository;
use crate::services::{Error, Result};

use super::SubroutinesService;

#[derive(Clone, Debug)]
pub struct SubroutineCreateInput {
    pub name: String,
    pub path: PathBuf,
    pub kind: SubroutineKind,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Create {
    async fn create(&self, input: SubroutineCreateInput) -> Result<Subroutine>;
}

#[async_trait]
impl<T> Create for SubroutinesService<T>
where
    T: SubroutineRepository,
{
    /// Creates a Subroutine entry in the repository.
    async fn create(&self, input: SubroutineCreateInput) -> Result<Subroutine> {
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

    use crate::{
        config::{fixtures::holodekk_config, HolodekkConfig},
        entities::{subroutine::fixtures::subroutine, Subroutine},
        repositories::{fixtures::subroutine_repository, MockSubroutineRepository},
        services::Error,
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn creates_subroutine(
        holodekk_config: HolodekkConfig,
        mut subroutine_repository: MockSubroutineRepository,
        subroutine: Subroutine,
    ) -> Result<()> {
        let input = SubroutineCreateInput {
            name: subroutine.name.clone(),
            path: subroutine.path.clone(),
            kind: subroutine.kind,
        };

        let sub_name = subroutine.name.clone();
        subroutine_repository
            .expect_subroutine_get_by_name()
            .withf(move |name, _inc| name == &sub_name)
            .return_const(Err(crate::repositories::Error::NotFound));

        let sub_path = subroutine.path.clone();
        let sub_name = subroutine.name.clone();

        subroutine_repository
            .expect_subroutine_create()
            .withf(move |new_sub: &Subroutine| {
                (*new_sub).path.eq(&sub_path) && (*new_sub).name.eq(&sub_name)
            })
            .return_const(Ok(subroutine.clone()));

        let service = SubroutinesService::new(
            Arc::new(holodekk_config),
            Arc::new(subroutine_repository),
            "test-namespace",
        );

        let sub = service.create(input).await?;
        assert_eq!(&sub, &subroutine);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn rejects_duplicate_subroutine_name(
        holodekk_config: HolodekkConfig,
        mut subroutine_repository: MockSubroutineRepository,
        subroutine: Subroutine,
    ) {
        let input = SubroutineCreateInput {
            name: subroutine.name.clone(),
            path: subroutine.path.clone(),
            kind: subroutine.kind,
        };

        let sub_name = subroutine.name.clone();

        subroutine_repository
            .expect_subroutine_get_by_name()
            .withf(move |name, _inc| name == &sub_name)
            .return_const(Ok(subroutine.to_owned()));

        let service = SubroutinesService::new(
            Arc::new(holodekk_config),
            Arc::new(subroutine_repository),
            "test-namespace",
        );

        let res = service.create(input).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::Duplicate);
    }
}
