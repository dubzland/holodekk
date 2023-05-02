use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use log::error;
#[cfg(test)]
use mockall::automock;

use crate::services::EntityServiceError;

#[cfg_attr(test, automock)]
pub trait ApiState<S1, S2>: Send + Sync + 'static
where
    S1: Send + Sync + 'static,
    S2: Send + Sync + 'static,
{
    fn scene_entity_service(&self) -> Arc<S1>;
    fn subroutine_entity_service(&self) -> Arc<S2>;
}

impl IntoResponse for EntityServiceError {
    fn into_response(self) -> Response {
        match self {
            EntityServiceError::NotUnique(_) => (StatusCode::CONFLICT, self.to_string()),
            EntityServiceError::NotFound(_)
            | EntityServiceError::InvalidEntityId(_)
            | EntityServiceError::InvalidImageId(_) => (StatusCode::NOT_FOUND, self.to_string()),
            EntityServiceError::Repository(err) => {
                error!("Repository error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
            EntityServiceError::Unexpected(err) => {
                error!("Unexpected error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
        }
        .into_response()
    }
}

pub mod commands;
pub mod routers;
