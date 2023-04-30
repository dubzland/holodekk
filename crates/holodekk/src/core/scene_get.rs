use std::sync::Arc;

use log::trace;

use crate::core::{
    entities::{SceneEntity, SceneEntityId},
    repositories::{self, ScenesRepository},
};

#[derive(Clone, Debug)]
pub struct Request<'a> {
    pub id: &'a SceneEntityId,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Scene not found with id {0}")]
    NotFound(SceneEntityId),
    #[error("General repository error occurred")]
    Repository(#[from] repositories::Error),
}

pub type Result = std::result::Result<SceneEntity, Error>;

pub async fn execute<R>(repo: Arc<R>, request: Request<'_>) -> Result
where
    R: ScenesRepository,
{
    trace!("get_scene:execute({:?})", request);

    let scene = repo.scenes_get(request.id).await.map_err(|err| match err {
        repositories::Error::NotFound(id) => Error::NotFound(id),
        _ => Error::from(err),
    })?;

    Ok(scene)
}
