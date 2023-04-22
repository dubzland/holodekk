mod create;
pub use create::*;

mod delete;
pub use delete::*;

mod find;
pub use find::*;

mod get;
pub use get::*;

use std::sync::Arc;

use crate::servers::director::DirectorRequest;

use super::repositories::ProjectorsRepository;

/// Service object for managing [ProjectorEntity](super::entities::ProjectorEntity) instances.
#[derive(Debug)]
pub struct ProjectorsService<R>
where
    R: ProjectorsRepository,
{
    repo: Arc<R>,
    director: tokio::sync::mpsc::Sender<DirectorRequest>,
}

impl<R> ProjectorsService<R>
where
    R: ProjectorsRepository,
{
    pub fn new(repo: Arc<R>, director: tokio::sync::mpsc::Sender<DirectorRequest>) -> Self {
        Self { repo, director }
    }

    pub fn director(&self) -> tokio::sync::mpsc::Sender<DirectorRequest> {
        self.director.clone()
    }
}
