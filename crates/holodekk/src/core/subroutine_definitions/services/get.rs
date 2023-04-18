use async_trait::async_trait;

use crate::core::subroutine_definitions::{
    entities::SubroutineDefinition, GetSubroutineDefinition, Result, SubroutineDefinitionsError,
    SubroutineDefinitionsGetInput,
};

use super::SubroutineDefinitionsService;

#[async_trait]
impl GetSubroutineDefinition for SubroutineDefinitionsService {
    async fn get<'a>(
        &self,
        input: &'a SubroutineDefinitionsGetInput,
    ) -> Result<SubroutineDefinition> {
        let definitions = self.definitions.read().unwrap();
        if let Some(definition) = definitions.get(input.id()) {
            Ok(definition.to_owned())
        } else {
            Err(SubroutineDefinitionsError::NotFound(input.id().into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::RwLock;

    use rstest::*;

    use crate::core::subroutine_definitions::{
        entities::{fixtures::subroutine_definition, SubroutineDefinition},
        SubroutineDefinitionsError,
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_definition_for_existing_subroutine_definition(
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        let mut definitions = HashMap::new();
        definitions.insert(
            subroutine_definition.id().to_string(),
            subroutine_definition.clone(),
        );

        let service = SubroutineDefinitionsService::new(RwLock::new(definitions));

        assert_eq!(
            service
                .get(&SubroutineDefinitionsGetInput::new(
                    subroutine_definition.id()
                ))
                .await?,
            subroutine_definition
        );
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_nonexisting_subroutine_definition() -> Result<()> {
        let definitions = HashMap::new();

        let service = SubroutineDefinitionsService::new(RwLock::new(definitions));

        assert!(matches!(
            service
                .get(&SubroutineDefinitionsGetInput::new("nonexistent"))
                .await
                .unwrap_err(),
            SubroutineDefinitionsError::NotFound(..)
        ));
        Ok(())
    }
}
