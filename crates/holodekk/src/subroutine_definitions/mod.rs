mod create;
mod find;
mod get;

use std::path::PathBuf;

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::core::entities::SubroutineDefinitionEntity;
use crate::core::enums::SubroutineKind;

#[derive(thiserror::Error, Debug)]
pub enum CreateSubroutineDefinitionError {
    #[error("A subroutine definition with the given name already exists: {0}")]
    Conflict(String),
}

#[derive(thiserror::Error, Debug)]
pub enum FindSubroutineDefinitionsError {}

#[derive(thiserror::Error, Debug)]
pub enum GetSubroutineDefinitionError {
    #[error("Subroutine definition not found matching id: {0}")]
    NotFound(String),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CreateSubroutineDefinition {
    async fn create_subroutine_definition<'a>(
        &self,
        name: &'a str,
        path: &'a PathBuf,
        kind: SubroutineKind,
    ) -> std::result::Result<SubroutineDefinitionEntity, CreateSubroutineDefinitionError>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait FindSubroutineDefinitions {
    async fn find_subroutine_definitions<'a>(
        &self,
        name: Option<&'a str>,
        path: Option<&'a PathBuf>,
        input: Option<SubroutineKind>,
    ) -> std::result::Result<Vec<SubroutineDefinitionEntity>, FindSubroutineDefinitionsError>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetSubroutineDefinition {
    async fn get_subroutine_definition<'a>(
        &self,
        id: &'a str,
    ) -> std::result::Result<SubroutineDefinitionEntity, GetSubroutineDefinitionError>;
}

pub trait SubroutineDefinitionMethods:
    CreateSubroutineDefinition
    + FindSubroutineDefinitions
    // + GetSubroutineDefinition
    + Send
    + Sync
    + 'static
{
}

impl<T> SubroutineDefinitionMethods for T where
    T: CreateSubroutineDefinition
        + FindSubroutineDefinitions
        // + GetSubroutineDefinition
        + Send
        + Sync
        + 'static
{
}

// pub async fn create_service<C>(config: Arc<C>) -> Result<services::SubroutineDefinitionsService>
// where
//     C: HolodekkConfig,
// {
//     let definitions = init::initialize_subroutine_definitions(config)?;

//     Ok(services::SubroutineDefinitionsService::new(
//         std::sync::RwLock::new(definitions),
//     ))
// }

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use super::*;

    #[fixture]
    pub fn mock_create_subroutine_definition() -> MockCreateSubroutineDefinition {
        MockCreateSubroutineDefinition::default()
    }
}
