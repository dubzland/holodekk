use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use log::error;
#[cfg(test)]
use mockall::automock;

use crate::core::services::scene::Error;

#[cfg_attr(test, automock)]
pub trait ApiState<S>: Send + Sync + 'static
where
    S: Send + Sync + 'static,
{
    fn scenes_service(&self) -> Arc<S>;
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::NotUnique(_) => (StatusCode::CONFLICT, self.to_string()),
            Error::NotFound(_) | Error::InvalidId(_) => (StatusCode::NOT_FOUND, self.to_string()),
            Error::Repository(err) => {
                error!("Repository error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
            Error::Unexpected(err) => {
                error!("Unexpected error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
        }
        .into_response()
    }
}

pub mod commands;
pub mod routers;
