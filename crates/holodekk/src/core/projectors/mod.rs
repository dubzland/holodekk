pub mod api;
pub mod entities;
mod init;
pub mod repositories;
pub mod services;
pub mod worker;

use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall::*;

use crate::config::HolodekkConfig;
use crate::core::services::ServiceStop;

use entities::Projector;

pub type Result<T> = std::result::Result<T, ProjectorsError>;

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum ProjectorsError {
    #[error("Projector {0} not found")]
    NotFound(String),
    #[error("Projector with id {0} is already running")]
    AlreadyRunning(String),
    #[error("Repository error occurred")]
    Repository(#[from] crate::core::repositories::RepositoryError),
    #[error("Error occurred during projector spawn")]
    Spawn(String),
    #[error("Error occurred during projector shutdown")]
    Shutdown(String),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CreateProjector {
    async fn create<'a>(&self, input: &'a ProjectorsCreateInput<'a>) -> Result<Projector>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait DeleteProjector {
    async fn delete<'a>(&self, input: &'a ProjectorsDeleteInput<'a>) -> Result<()>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait FindProjectors {
    async fn find<'a>(&self, input: &'a ProjectorsFindInput<'a>) -> Vec<Projector>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetProjector {
    async fn get<'a>(&self, input: &'a ProjectorsGetInput<'a>) -> Result<Projector>;
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
    fleet: Option<&'f str>,
    namespace: Option<&'f str>,
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

pub async fn create_service<C, R>(
    config: Arc<C>,
    repo: Arc<R>,
) -> Result<
    impl CreateProjector + DeleteProjector + FindProjectors + GetProjector + ServiceStop + Send + Sync,
>
where
    C: HolodekkConfig,
    R: repositories::ProjectorsRepository + 'static,
{
    init::initialize_projectors(config.clone(), repo.clone()).await?;

    let worker = worker::start_worker(config.clone());

    Ok(services::ProjectorsService::new(config, repo, worker))
}

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use super::*;

    #[fixture]
    pub fn mock_get_projector() -> MockGetProjector {
        MockGetProjector::default()
    }
}
