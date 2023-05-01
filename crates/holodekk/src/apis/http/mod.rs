use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use log::error;
#[cfg(test)]
use mockall::automock;

use crate::core::services::Error;

#[cfg_attr(test, automock)]
pub trait ApiState<S1, S2>: Send + Sync + 'static
where
    S1: Send + Sync + 'static,
    S2: Send + Sync + 'static,
{
    fn scenes_service(&self) -> Arc<S1>;
    fn subroutines_service(&self) -> Arc<S2>;
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::NotUnique(_) => (StatusCode::CONFLICT, self.to_string()),
            Error::NotFound(_) | Error::InvalidEntityId(_) | Error::InvalidImageId(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
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
