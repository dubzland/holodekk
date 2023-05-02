use std::path::PathBuf;

use async_trait::async_trait;
use log::trace;

use crate::core::{entities::SubroutineDefinitionEntity, enums::SubroutineKind};
use crate::HolodekkServices;

use super::{CreateSubroutineDefinition, CreateSubroutineDefinitionError};

#[async_trait]
impl<R> CreateSubroutineDefinition for HolodekkServices<R>
where
    R: Send + Sync,
{
    async fn create_subroutine_definition<'a>(
        &self,
        name: &'a str,
        path: &'a PathBuf,
        kind: SubroutineKind,
    ) -> std::result::Result<SubroutineDefinitionEntity, CreateSubroutineDefinitionError> {
        // make sure this subroutine does not already exist
        trace!("Checking for subroutine definition with name: {}", name,);
        if self.definitions.read().unwrap().contains_key(name) {
            Err(CreateSubroutineDefinitionError::Conflict(name.into()))
        } else {
            let definition = SubroutineDefinitionEntity::new(name, path, kind);
            self.definitions
                .write()
                .unwrap()
                .insert(definition.name().to_string(), definition.clone());
            Ok(definition)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rstest::*;

    use crate::core::{
        entities::{fixtures::mock_subroutine_definition, SubroutineDefinitionEntity},
        repositories::fixtures::{mock_repository, MockRepository},
    };
    use crate::fixtures::mock_holodekk_service;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn rejects_duplicate_subroutine_name(
        mock_subroutine_definition: SubroutineDefinitionEntity,
        mock_repository: MockRepository,
    ) {
        let mut definitions = HashMap::new();
        definitions.insert(
            mock_subroutine_definition.name().to_string(),
            mock_subroutine_definition.clone(),
        );

        let (_, _, service) = mock_holodekk_service(mock_repository, definitions);

        let res = service
            .create_subroutine_definition(
                mock_subroutine_definition.name(),
                mock_subroutine_definition.path(),
                mock_subroutine_definition.kind(),
            )
            .await;
        assert!(matches!(
            res.unwrap_err(),
            CreateSubroutineDefinitionError::Conflict(..)
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn creates_subroutine_definition(
        mock_repository: MockRepository,
        mock_subroutine_definition: SubroutineDefinitionEntity,
    ) {
        let definitions = HashMap::new();
        let (_, _, service) = mock_holodekk_service(mock_repository, definitions);
        let definition = service
            .create_subroutine_definition(
                mock_subroutine_definition.name(),
                mock_subroutine_definition.path(),
                mock_subroutine_definition.kind(),
            )
            .await
            .unwrap();

        assert_eq!(definition, mock_subroutine_definition);
    }
}
