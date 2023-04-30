use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use log::error;

use holodekk::core::actions::{scene_create, scene_delete, scenes_find};
use holodekk::core::entities::EntityIdError;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Invalid Resource Id")]
    InvalidResourceId(#[from] EntityIdError),
    #[error("Scene already exists")]
    SceneConflict {
        #[source]
        source: scene_create::Error,
    },
    #[error("Scene not found")]
    SceneNotFound(String),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<scene_create::Error> for ApiError {
    fn from(err: scene_create::Error) -> Self {
        match err {
            scene_create::Error::Conflict(_) => ApiError::SceneConflict { source: err },
            _ => ApiError::from(anyhow::anyhow!("Unexpected error occurred: {}", err)),
        }
    }
}

impl From<scene_delete::Error> for ApiError {
    fn from(err: scene_delete::Error) -> Self {
        match err {
            scene_delete::Error::NotFound(id) => ApiError::SceneNotFound(id.to_string()),
            _ => ApiError::from(anyhow::anyhow!("Unexpected error occurred: {}", err)),
        }
    }
}

impl From<scenes_find::Error> for ApiError {
    fn from(err: scenes_find::Error) -> Self {
        ApiError::from(anyhow::anyhow!("Unexpected error occurred: {}", err))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        error!("Error encountered: {:?}", self);
        let code = match self {
            ApiError::InvalidResourceId(_) => StatusCode::NOT_FOUND,
            ApiError::SceneConflict { .. } => StatusCode::CONFLICT,
            ApiError::SceneNotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let response = (code, format!("{:?}", self));
        response.into_response()
    }
}
