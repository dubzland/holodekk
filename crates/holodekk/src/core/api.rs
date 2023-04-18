use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use super::projectors::ProjectorsError;
use super::subroutine_definitions::SubroutineDefinitionsError;
use super::subroutines::SubroutinesError;

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum ApiError {
    #[error("Unexpected projector error occurred")]
    Projector(#[from] ProjectorsError),
    #[error("Unexpected subroutine error occurred")]
    Subroutine(#[from] SubroutinesError),
    #[error("Unexpected subroutine definition error occurred")]
    SubroutineDefinition(#[from] SubroutineDefinitionsError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let response = match self {
            ApiError::Projector(err) => match err {
                ProjectorsError::AlreadyRunning(id) => (
                    StatusCode::CONFLICT,
                    format!("Projector already running with id {}", id),
                ),
                ProjectorsError::NotFound(id) => (
                    StatusCode::NOT_FOUND,
                    format!("Could not find a projector with id {}", id),
                ),
                err => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unexpected projector error occurred: {}", err),
                ),
            },
            ApiError::Subroutine(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unexpected subroutine error occurred: {}", err),
            ),
            ApiError::SubroutineDefinition(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unexpected subroutine definition error occurred: {}", err),
            ),
        };
        response.into_response()
    }
}
