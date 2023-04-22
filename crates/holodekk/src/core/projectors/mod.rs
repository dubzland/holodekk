//! Projector instances running on the Holodekk.
//!
//! A projector is a background process, acting as a mediator between
//! [Subroutines](holodekk::core::subroutines) and the container engine.
pub mod entities;
pub mod repositories;
pub mod services;
pub mod worker;

use async_trait::async_trait;
#[cfg(test)]
use mockall::*;

use entities::ProjectorEntity;

#[derive(thiserror::Error, Debug)]
pub enum ProjectorsError {
    #[error("Projector {0} not found")]
    NotFound(String),
    #[error("Projector with id {0} is already running")]
    AlreadyRunning(String),
    #[error("Repository error occurred")]
    Repository(#[from] crate::repositories::RepositoryError),
    #[error("Error occurred during projector spawn")]
    Spawn(#[from] worker::SpawnError),
    #[error("Error occurred during projector termination")]
    Termination(#[from] worker::TerminationError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, ProjectorsError>;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CreateProjector {
    async fn create<'a>(&self, input: &'a ProjectorsCreateInput<'a>) -> Result<ProjectorEntity>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait DeleteProjector {
    async fn delete<'a>(&self, input: &'a ProjectorsDeleteInput<'a>) -> Result<()>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait FindProjectors {
    async fn find<'a>(&self, input: &'a ProjectorsFindInput<'a>) -> Result<Vec<ProjectorEntity>>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetProjector {
    async fn get<'a>(&self, input: &'a ProjectorsGetInput<'a>) -> Result<ProjectorEntity>;
}

#[derive(Clone, Debug)]
pub struct ProjectorsCreateInput<'c> {
    namespace: &'c str,
}

impl<'c> ProjectorsCreateInput<'c> {
    pub fn new(namespace: &'c str) -> Self {
        Self { namespace }
    }

    pub fn namespace(&self) -> &str {
        self.namespace
    }
}

#[derive(Clone, Debug)]
pub struct ProjectorsDeleteInput<'d> {
    id: &'d str,
}

impl<'d> ProjectorsDeleteInput<'d> {
    pub fn new(id: &'d str) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &str {
        self.id
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct ProjectorsFindInput<'f> {
    namespace: Option<&'f str>,
}

impl<'f> ProjectorsFindInput<'f> {
    pub fn new(namespace: Option<&'f str>) -> Self {
        Self { namespace }
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace
    }
}

#[derive(Clone, Debug)]
pub struct ProjectorsGetInput<'g> {
    id: &'g str,
}

impl<'g> ProjectorsGetInput<'g> {
    pub fn new(id: &'g str) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &str {
        self.id
    }
}

pub trait ProjectorsServiceMethods:
    CreateProjector + DeleteProjector + FindProjectors + GetProjector + Send + Sync + 'static
{
}
impl<T> ProjectorsServiceMethods for T where
    T: CreateProjector + DeleteProjector + FindProjectors + GetProjector + Send + Sync + 'static
{
}

#[cfg(test)]
pub mod fixtures {
    use mockall::mock;
    use rstest::*;

    use super::*;

    mock! {
        pub ProjectorsService {}
        #[async_trait]
        impl CreateProjector for ProjectorsService {
            async fn create<'a>(&self, input: &'a ProjectorsCreateInput<'a>) -> Result<ProjectorEntity>;
        }

        #[async_trait]
        impl DeleteProjector for ProjectorsService {
            async fn delete<'a>(&self, input: &'a ProjectorsDeleteInput<'a>) -> Result<()>;
        }

        #[async_trait]
        impl FindProjectors for ProjectorsService {
            async fn find<'a>(&self, input: &'a ProjectorsFindInput<'a>) -> Result<Vec<ProjectorEntity>>;
        }

        #[async_trait]
        impl GetProjector for ProjectorsService {
            async fn get<'a>(&self, input: &'a ProjectorsGetInput<'a>) -> Result<ProjectorEntity>;
        }
    }

    #[fixture]
    pub fn mock_projectors_service() -> MockProjectorsService {
        MockProjectorsService::default()
    }
}
