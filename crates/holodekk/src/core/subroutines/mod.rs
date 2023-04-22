pub mod entities;
pub mod repositories;
pub mod services;
pub mod worker;

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;

use crate::config::HolodekkConfig;

use entities::SubroutineEntity;

pub type Result<T> = std::result::Result<T, SubroutinesError>;

#[derive(thiserror::Error, Debug)]
pub enum SubroutinesError {
    #[error("Subroutine {0} not found")]
    NotFound(String),
    #[error("Subroutine already running")]
    AlreadyRunning,
    #[error("Repository error occurred")]
    Repository(#[from] crate::repositories::RepositoryError),
    #[error("Invalid subroutine defintion id: {0}")]
    InvalidSubroutineDefinition(String),
    #[error("Invalid projector id: {0}")]
    InvalidProjector(String),
    #[error("Failed to spawn subroutine")]
    Spawn(#[from] worker::SpawnError),
    #[error("Error occurred during subroutine shutdown")]
    Termination(#[from] worker::TerminationError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CreateSubroutine {
    async fn create<'c>(&self, input: &'c SubroutinesCreateInput<'c>) -> Result<SubroutineEntity>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait DeleteSubroutine {
    async fn delete<'c>(&self, input: &'c SubroutinesDeleteInput<'c>) -> Result<()>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait FindSubroutines {
    async fn find<'a>(&self, input: &'a SubroutinesFindInput<'a>) -> Result<Vec<SubroutineEntity>>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetSubroutine {
    async fn get<'c>(&self, input: &'c SubroutinesGetInput<'c>) -> Result<SubroutineEntity>;
}

#[derive(Clone, Debug)]
pub struct SubroutinesCreateInput<'c> {
    projector_id: &'c str,
    subroutine_definition_id: &'c str,
}

impl<'c> SubroutinesCreateInput<'c> {
    pub fn new(projector_id: &'c str, subroutine_definition_id: &'c str) -> Self {
        Self {
            projector_id,
            subroutine_definition_id,
        }
    }

    pub fn projector_id(&self) -> &str {
        self.projector_id
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
        self.id
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubroutinesFindInput<'f> {
    projector_id: Option<&'f str>,
    subroutine_definition_id: Option<&'f str>,
}

impl<'f> SubroutinesFindInput<'f> {
    pub fn new(projector_id: Option<&'f str>, subroutine_definition_id: Option<&'f str>) -> Self {
        Self {
            projector_id,
            subroutine_definition_id,
        }
    }

    pub fn projector_id(&self) -> Option<&str> {
        self.projector_id
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
        self.id
    }
}

pub trait SubroutinesServiceMethods:
    CreateSubroutine + DeleteSubroutine + FindSubroutines + GetSubroutine + Send + Sync + 'static
{
}
impl<T> SubroutinesServiceMethods for T where
    T: CreateSubroutine
        + DeleteSubroutine
        + FindSubroutines
        + GetSubroutine
        + Send
        + Sync
        + 'static
{
}

#[derive(Debug)]
pub struct SubroutinePaths {
    root: PathBuf,
    pidfile: PathBuf,
    logfile: PathBuf,
    socket: PathBuf,
}

impl SubroutinePaths {
    pub fn build<C>(config: Arc<C>, subroutine: &SubroutineEntity) -> Self
    where
        C: HolodekkConfig,
    {
        let mut root = config.subroutines_root().clone();
        root.push(subroutine.id());

        let mut pidfile = root.clone();
        pidfile.push("subroutine.pid");

        let mut logfile = root.clone();
        logfile.push("subroutine.log");

        let mut socket = root.clone();
        socket.push("log.sock");

        Self {
            root,
            pidfile,
            logfile,
            socket,
        }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn pidfile(&self) -> &PathBuf {
        &self.pidfile
    }

    pub fn logfile(&self) -> &PathBuf {
        &self.logfile
    }

    pub fn socket(&self) -> &PathBuf {
        &self.socket
    }
}

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use super::*;

    #[fixture]
    pub fn mock_create_subroutine() -> MockCreateSubroutine {
        MockCreateSubroutine::default()
    }

    #[fixture]
    pub fn mock_delete_subroutine() -> MockDeleteSubroutine {
        MockDeleteSubroutine::default()
    }

    #[fixture]
    pub fn mock_find_subroutines() -> MockFindSubroutines {
        MockFindSubroutines::default()
    }
}
