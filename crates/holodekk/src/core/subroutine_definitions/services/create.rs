use async_trait::async_trait;

use crate::core::services::{Error, Result};
use crate::core::subroutine_definitions::{
    entities::SubroutineDefinition, CreateSubroutineDefinition, SubroutineDefinitionsCreateInput,
};

use super::SubroutineDefinitionsService;

#[async_trait]
impl CreateSubroutineDefinition for SubroutineDefinitionsService {
    /// Creates a Subroutine entry in the repository.
    async fn create<'a>(
        &self,
        input: &'a SubroutineDefinitionsCreateInput<'a>,
    ) -> Result<SubroutineDefinition> {
        // make sure this subroutine does not already exist
        println!("Checking for subroutine with name: {}", input.name,);
        if self.definitions.read().unwrap().contains_key(input.name()) {
            return Err(Error::Duplicate);
        }

        let definition = SubroutineDefinition::new(input.name(), input.path(), input.kind);
        self.definitions
            .write()
            .unwrap()
            .insert(definition.name().to_string(), definition.clone());
        Ok(definition)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::RwLock;

    use rstest::*;

    use crate::core::services::Error;
    use crate::core::subroutine_definitions::entities::{
        fixtures::subroutine_definition, SubroutineDefinition,
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn creates_subroutine_definition(
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        let input = SubroutineDefinitionsCreateInput::new(
            subroutine_definition.name(),
            subroutine_definition.path(),
            subroutine_definition.kind(),
        );

        let definitions = RwLock::new(HashMap::new());
        let service = SubroutineDefinitionsService { definitions };
        let definition = service.create(&input).await?;

        assert_eq!(definition, subroutine_definition);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn returns_subroutine_definition(
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        let input = SubroutineDefinitionsCreateInput::new(
            subroutine_definition.name(),
            subroutine_definition.path(),
            subroutine_definition.kind(),
        );

        let definitions = HashMap::new();
        let service = SubroutineDefinitionsService {
            definitions: RwLock::new(definitions.clone()),
        };
        let def = service.create(&input).await?;

        assert_eq!(def, subroutine_definition);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn rejects_duplicate_subroutine_name(
        subroutine_definition: SubroutineDefinition,
    ) -> Result<()> {
        let input = SubroutineDefinitionsCreateInput::new(
            subroutine_definition.name(),
            subroutine_definition.path(),
            subroutine_definition.kind(),
        );

        let mut definitions = HashMap::new();
        definitions.insert(input.name().to_string(), subroutine_definition.clone());
        let service = SubroutineDefinitionsService {
            definitions: RwLock::new(definitions.clone()),
        };
        let res = service.create(&input).await;
        assert!(matches!(res.unwrap_err(), Error::Duplicate));
        Ok(())
    }
}
