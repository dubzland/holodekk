use std::path::PathBuf;

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::core::{
    entities::{SubroutineDefinition, SubroutineKind},
    repositories::{subroutine_definition_repo_id, SubroutineDefinitionsRepository},
    services::{Error, Result},
};

use super::SubroutineDefinitionsService;

#[derive(Clone, Debug)]
pub struct SubroutineDefinitionsCreateInput {
    pub name: String,
    pub path: PathBuf,
    pub kind: SubroutineKind,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Create {
    async fn create(&self, input: SubroutineDefinitionsCreateInput)
        -> Result<SubroutineDefinition>;
}

#[async_trait]
impl<T> Create for SubroutineDefinitionsService<T>
where
    T: SubroutineDefinitionsRepository,
{
    /// Creates a Subroutine entry in the repository.
    async fn create(
        &self,
        input: SubroutineDefinitionsCreateInput,
    ) -> Result<SubroutineDefinition> {
        // make sure this subroutine does not already exist
        println!("Checking for subroutine with name: {}", input.name,);
        if self
            .repo
            .subroutine_definitions_get(&subroutine_definition_repo_id(&input.name))
            .await
            .is_ok()
        {
            return Err(Error::Duplicate);
        }

        let subroutine = SubroutineDefinition::new(input.name, input.path, input.kind);
        let subroutine = self.repo.subroutine_definitions_create(subroutine).await?;
        Ok(subroutine)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::core::{
        entities::{subroutine::definition::fixtures::subroutine_definition, SubroutineDefinition},
        repositories::{
            self, fixtures::subroutine_definitions_repository, MockSubroutineDefinitionsRepository,
        },
        services::Error,
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn creates_subroutine_definition(
        mut subroutine_definitions_repository: MockSubroutineDefinitionsRepository,
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        let input = SubroutineDefinitionsCreateInput {
            name: subroutine_definition.name.clone(),
            path: subroutine_definition.path.clone(),
            kind: subroutine_definition.kind,
        };

        let sub_name = subroutine_definition.name.clone();
        subroutine_definitions_repository
            .expect_subroutine_definitions_get()
            .withf(move |name| name == &subroutine_definition_repo_id(&sub_name))
            .return_const(Err(repositories::Error::NotFound));

        let sub_path = subroutine_definition.path.clone();
        let sub_name = subroutine_definition.name.clone();

        subroutine_definitions_repository
            .expect_subroutine_definitions_create()
            .withf(move |new_sub: &SubroutineDefinition| {
                (*new_sub).path.eq(&sub_path) && (*new_sub).name.eq(&sub_name)
            })
            .return_const(Ok(subroutine_definition.clone()));

        let service =
            SubroutineDefinitionsService::new(Arc::new(subroutine_definitions_repository));

        let def = service.create(input).await?;
        assert_eq!(&def, &subroutine_definition);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn rejects_duplicate_subroutine_name(
        mut subroutine_definitions_repository: MockSubroutineDefinitionsRepository,
        subroutine_definition: SubroutineDefinition,
    ) {
        let input = SubroutineDefinitionsCreateInput {
            name: subroutine_definition.name.clone(),
            path: subroutine_definition.path.clone(),
            kind: subroutine_definition.kind,
        };

        let sub_name = subroutine_definition.name.clone();

        subroutine_definitions_repository
            .expect_subroutine_definitions_get()
            .withf(move |name| name == &subroutine_definition_repo_id(&sub_name))
            .return_const(Ok(subroutine_definition.to_owned()));

        let service =
            SubroutineDefinitionsService::new(Arc::new(subroutine_definitions_repository));

        let res = service.create(input).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::Duplicate);
    }
}
