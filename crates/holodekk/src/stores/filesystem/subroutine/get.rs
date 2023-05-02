use async_trait::async_trait;

use crate::core::entities::SubroutineDefinitionEntity;
use crate::HolodekkServices;

use super::{GetSubroutineDefinition, GetSubroutineDefinitionError};

#[async_trait]
impl<R> GetSubroutineDefinition for HolodekkServices<R>
where
    R: Send + Sync,
{
    async fn get_subroutine_definition<'a>(
        &self,
        id: &'a str,
    ) -> std::result::Result<SubroutineDefinitionEntity, GetSubroutineDefinitionError> {
        let definitions = self.definitions.read().unwrap();
        if let Some(definition) = definitions.get(id) {
            Ok(definition.to_owned())
        } else {
            Err(GetSubroutineDefinitionError::NotFound(id.into()))
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
    async fn returns_error_for_nonexisting_subroutine_definition(mock_repository: MockRepository) {
        let definitions = HashMap::new();

        let (_, _, service) = mock_holodekk_service(mock_repository, definitions);

        assert!(matches!(
            service
                .get_subroutine_definition("nonexistent")
                .await
                .unwrap_err(),
            GetSubroutineDefinitionError::NotFound(..)
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn returns_definition_for_existing_subroutine_definition(
        mock_repository: MockRepository,
        mock_subroutine_definition: SubroutineDefinitionEntity,
    ) {
        let mut definitions = HashMap::new();
        definitions.insert(
            mock_subroutine_definition.id().to_string(),
            mock_subroutine_definition.clone(),
        );
        let (_, _, service) = mock_holodekk_service(mock_repository, definitions);

        assert_eq!(
            service
                .get_subroutine_definition(mock_subroutine_definition.id())
                .await
                .unwrap(),
            mock_subroutine_definition
        );
    }
}
