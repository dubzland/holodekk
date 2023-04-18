pub mod api;
pub mod entities;
mod init;
pub mod repositories;
pub mod services;

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::config::HolodekkConfig;
use crate::core::subroutine_definitions::entities::{SubroutineDefinition, SubroutineKind};

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
    ) -> Result<SubroutineDefinition>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetSubroutineDefinition {
    async fn get<'a>(
        &self,
        input: &'a SubroutineDefinitionsGetInput<'a>,
    ) -> Result<SubroutineDefinition>;
}

pub trait SubroutineDefinitionsServiceMethods:
    CreateSubroutineDefinition + GetSubroutineDefinition + Send + Sync + 'static
{
}
impl<T> SubroutineDefinitionsServiceMethods for T where
    T: CreateSubroutineDefinition + GetSubroutineDefinition + Send + Sync + 'static
{
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

pub async fn create_service<C>(config: Arc<C>) -> Result<services::SubroutineDefinitionsService>
where
    C: HolodekkConfig,
{
    let definitions = init::initialize_subroutine_definitions(config)?;

    Ok(services::SubroutineDefinitionsService::new(
        std::sync::RwLock::new(definitions),
    ))
}

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
            ) -> Result<SubroutineDefinition>;
        }
        #[async_trait]
        impl GetSubroutineDefinition for SubroutineDefinitionsService {
            async fn get<'a>(
                &self,
                input: &'a SubroutineDefinitionsGetInput<'a>,
            ) -> Result<SubroutineDefinition>;
        }
    }

    #[fixture]
    pub fn mock_subroutine_definitions_service() -> MockSubroutineDefinitionsService {
        MockSubroutineDefinitionsService::default()
    }
}
