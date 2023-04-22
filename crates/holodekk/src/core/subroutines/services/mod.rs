mod create;
pub use create::*;

mod delete;
pub use delete::*;

mod find;
pub use find::*;

mod get;
pub use get::*;

use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use crate::servers::director::DirectorRequest;

use crate::core::projectors::{
    entities::ProjectorEntity, ProjectorsError, ProjectorsGetInput, ProjectorsServiceMethods,
};
use crate::core::subroutine_definitions::{
    entities::SubroutineDefinitionEntity, SubroutineDefinitionsError,
    SubroutineDefinitionsGetInput, SubroutineDefinitionsServiceMethods,
};

use super::repositories::SubroutinesRepository;
use super::{Result, SubroutinesError};

#[derive(Debug)]
pub struct SubroutinesService<R, P, D>
where
    R: SubroutinesRepository,
    P: ProjectorsServiceMethods,
    D: SubroutineDefinitionsServiceMethods,
{
    repo: Arc<R>,
    director: tokio::sync::mpsc::Sender<DirectorRequest>,
    projectors: Arc<P>,
    definitions: Arc<D>,
}

impl<R, P, D> SubroutinesService<R, P, D>
where
    R: SubroutinesRepository,
    P: ProjectorsServiceMethods,
    D: SubroutineDefinitionsServiceMethods,
{
    pub fn new(
        repo: Arc<R>,
        director: tokio::sync::mpsc::Sender<DirectorRequest>,
        projectors: Arc<P>,
        definitions: Arc<D>,
    ) -> Self {
        Self {
            repo,
            director,
            projectors,
            definitions,
        }
    }

    pub fn director(&self) -> Sender<DirectorRequest> {
        self.director.clone()
    }

    pub async fn get_projector(&self, id: &str) -> Result<ProjectorEntity> {
        let input = ProjectorsGetInput::new(id);
        self.projectors.get(&input).await.map_err(|err| match err {
            ProjectorsError::NotFound(_) => SubroutinesError::InvalidProjector(id.to_string()),
            err => SubroutinesError::from(anyhow::anyhow!(err.to_string())),
        })
    }

    pub async fn get_subroutine_definition(&self, id: &str) -> Result<SubroutineDefinitionEntity> {
        let input = SubroutineDefinitionsGetInput::new(id);
        self.definitions.get(&input).await.map_err(|err| match err {
            SubroutineDefinitionsError::NotFound(_) => {
                SubroutinesError::InvalidSubroutineDefinition(id.to_string())
            }
            err => SubroutinesError::from(anyhow::anyhow!(err.to_string())),
        })
    }
}
