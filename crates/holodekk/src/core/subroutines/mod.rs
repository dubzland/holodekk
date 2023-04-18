pub mod api;
pub mod entities;
mod init;
pub mod repositories;
pub mod services;
pub mod worker;

use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::config::HolodekkConfig;
use crate::core::services::ServiceStop;
use crate::core::subroutine_definitions::{
    services::SubroutineDefinitionsService, SubroutineDefinitionsError,
};

use entities::Subroutine;

pub type Result<T> = std::result::Result<T, SubroutinesError>;

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum SubroutinesError {
    #[error("Subroutine {0} not found")]
    NotFound(String),
    #[error("Subroutine with id {0} is already running")]
    AlreadyRunning(String),
    #[error("Repository error occurred")]
    Repository(#[from] crate::core::repositories::RepositoryError),
    #[error("Invalid subroutine defintion id: {0}")]
    InvalidSubroutineDefinition(String),
    #[error("Failed to spawn subroutine")]
    SpawnError(String),
    #[error("Unknown subroutine definition error")]
    UnexpectedSubroutineDefinitionError(#[from] SubroutineDefinitionsError),
    #[error("Error occurred during subroutine shutdown")]
    Shutdown(String),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CreateSubroutine {
    async fn create<'c>(&self, input: &'c SubroutinesCreateInput<'c>) -> Result<Subroutine>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait DeleteSubroutine {
    async fn delete<'c>(&self, input: &'c SubroutinesDeleteInput<'c>) -> Result<()>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait FindSubroutines {
    async fn find<'a>(&self, input: &'a SubroutinesFindInput<'a>) -> Result<Vec<Subroutine>>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetSubroutine {
    async fn get<'c>(&self, input: &'c SubroutinesGetInput<'c>) -> Result<Subroutine>;
}

#[derive(Clone, Debug)]
pub struct SubroutinesCreateInput<'c> {
    fleet: &'c str,
    namespace: &'c str,
    subroutine_definition_id: &'c str,
}

impl<'c> SubroutinesCreateInput<'c> {
    pub fn new(fleet: &'c str, namespace: &'c str, subroutine_definition_id: &'c str) -> Self {
        Self {
            fleet,
            namespace,
            subroutine_definition_id,
        }
    }

    pub fn fleet(&self) -> &str {
        self.fleet
    }

    pub fn namespace(&self) -> &str {
        self.namespace
    }

    pub fn subroutine_definition_id(&self) -> &str {
        self.subroutine_definition_id
    }
}

#[derive(Clone, Debug)]
pub struct SubroutinesDeleteInput<'c> {
    id: &'c str,
}

impl<'c> SubroutinesDeleteInput<'c> {
    pub fn new(id: &'c str) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone, Debug)]
pub struct SubroutinesFindInput<'f> {
    fleet: Option<&'f str>,
    namespace: Option<&'f str>,
    subroutine_definition_id: Option<&'f str>,
}

impl<'f> SubroutinesFindInput<'f> {
    pub fn new(
        fleet: Option<&'f str>,
        namespace: Option<&'f str>,
        subroutine_definition_id: Option<&'f str>,
    ) -> Self {
        Self {
            fleet,
            namespace,
            subroutine_definition_id,
        }
    }

    pub fn fleet(&self) -> Option<&str> {
        self.fleet
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace
    }

    pub fn subroutine_definition_id(&self) -> Option<&str> {
        self.subroutine_definition_id
    }
}

#[derive(Clone, Debug)]
pub struct SubroutinesGetInput<'c> {
    id: &'c str,
}

impl<'c> SubroutinesGetInput<'c> {
    pub fn new(id: &'c str) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

pub async fn create_service<C, R>(
    config: Arc<C>,
    definitions: Arc<SubroutineDefinitionsService>,
    repo: Arc<R>,
) -> Result<impl CreateSubroutine + FindSubroutines + ServiceStop + Send + Sync>
where
    C: HolodekkConfig,
    R: repositories::SubroutinesRepository + 'static,
{
    init::initialize_subroutines(config.clone(), repo.clone()).await?;

    let worker = worker::start_worker(config.clone());

    Ok(services::SubroutinesService::new(repo, definitions, worker))
}
