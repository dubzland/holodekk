pub mod entities;
pub mod repositories;
pub mod services;

use std::path::{Path, PathBuf};

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use entities::{SubroutineDefinitionEntity, SubroutineKind};

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum SubroutineDefinitionsError {
    #[error("Subroutine definition does not exist {0}")]
    NotFound(String),
    #[error("Subroutine definition already exists: {0}")]
    Duplicate(String),
}

pub type Result<T> = std::result::Result<T, SubroutineDefinitionsError>;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CreateSubroutineDefinition {
    async fn create<'a>(
        &self,
        input: &'a SubroutineDefinitionsCreateInput<'a>,
    ) -> Result<SubroutineDefinitionEntity>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait FindSubroutineDefinitions {
    async fn find<'a>(
        &self,
        input: &'a SubroutineDefinitionsFindInput<'a>,
    ) -> Result<Vec<SubroutineDefinitionEntity>>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetSubroutineDefinition {
    async fn get<'a>(
        &self,
        input: &'a SubroutineDefinitionsGetInput<'a>,
    ) -> Result<SubroutineDefinitionEntity>;
}

#[derive(Clone, Debug)]
pub struct SubroutineDefinitionsCreateInput<'c> {
    name: &'c str,
    path: &'c PathBuf,
    kind: SubroutineKind,
}

impl<'c> SubroutineDefinitionsCreateInput<'c> {
    pub fn new(name: &'c str, path: &'c PathBuf, kind: SubroutineKind) -> Self {
        Self { name, path, kind }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn path(&self) -> &PathBuf {
        self.path
    }

    pub fn kind(&self) -> SubroutineKind {
        self.kind
    }
}

#[derive(Clone, Debug)]
pub struct SubroutineDefinitionsFindInput<'f> {
    name: Option<&'f str>,
    path: Option<&'f Path>,
    kind: Option<SubroutineKind>,
}

impl<'f> SubroutineDefinitionsFindInput<'f> {
    pub fn new(
        name: Option<&'f str>,
        path: Option<&'f Path>,
        kind: Option<SubroutineKind>,
    ) -> Self {
        Self { name, path, kind }
    }

    pub fn name(&self) -> Option<&str> {
        self.name
    }

    pub fn path(&self) -> Option<&Path> {
        self.path
    }

    pub fn kind(&self) -> Option<SubroutineKind> {
        self.kind
    }
}

#[derive(Clone, Debug)]
pub struct SubroutineDefinitionsGetInput<'g> {
    id: &'g str,
}

impl<'g> SubroutineDefinitionsGetInput<'g> {
    pub fn new(id: &'g str) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &str {
        self.id
    }
}

pub trait SubroutineDefinitionsServiceMethods:
    CreateSubroutineDefinition
    + FindSubroutineDefinitions
    + GetSubroutineDefinition
    + Send
    + Sync
    + 'static
{
}

impl<T> SubroutineDefinitionsServiceMethods for T where
    T: CreateSubroutineDefinition
        + FindSubroutineDefinitions
        + GetSubroutineDefinition
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
    use mockall::mock;
    use rstest::*;

    use super::*;

    mock! {
        pub SubroutineDefinitionsService {}
        #[async_trait]
        impl CreateSubroutineDefinition for SubroutineDefinitionsService {
            async fn create<'a>(
                &self,
                input: &'a SubroutineDefinitionsCreateInput<'a>,
            ) -> Result<SubroutineDefinitionEntity>;
        }
        #[async_trait]
        impl FindSubroutineDefinitions for SubroutineDefinitionsService {
            async fn find<'a>(
                &self,
                input: &'a SubroutineDefinitionsFindInput<'a>,
            ) -> Result<Vec<SubroutineDefinitionEntity>>;
        }
        #[async_trait]
        impl GetSubroutineDefinition for SubroutineDefinitionsService {
            async fn get<'a>(
                &self,
                input: &'a SubroutineDefinitionsGetInput<'a>,
            ) -> Result<SubroutineDefinitionEntity>;
        }
    }

    #[fixture]
    pub fn mock_subroutine_definitions_service() -> MockSubroutineDefinitionsService {
        MockSubroutineDefinitionsService::default()
    }
}
