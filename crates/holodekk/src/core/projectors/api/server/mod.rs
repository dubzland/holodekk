mod create;
mod list;
mod stop;

use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};

use crate::core::projectors::services::{
    CreateProjector, DeleteProjector, FindProjectors, GetProjector,
};
use crate::core::services::Error;
use crate::core::subroutine_definitions::services::CreateSubroutineDefinition;

pub struct ApiServices<P, D> {
    projectors_service: Arc<P>,
    definitions_service: Arc<D>,
}

impl<P, D> ApiServices<P, D> {
    pub fn projectors(&self) -> Arc<P> {
        self.projectors_service.clone()
    }

    pub fn definitions(&self) -> Arc<D> {
        self.definitions_service.clone()
    }
}

pub fn router<P, D>(projectors_service: Arc<P>, definitions_service: Arc<D>) -> axum::Router
where
    P: CreateProjector + DeleteProjector + FindProjectors + GetProjector + Send + Sync + 'static,
    D: CreateSubroutineDefinition + Send + Sync + 'static,
{
    // Create the global services
    let services = Arc::new(ApiServices {
        projectors_service,
        definitions_service,
    });

    Router::new()
        .route("/", get(list::handler))
        .route("/", post(create::handler))
        .route("/:id", delete(stop::handler))
        .with_state(services)
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let response = match self {
            Error::Duplicate => (
                StatusCode::CONFLICT,
                "Projector already running for specified namespace",
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error"),
        };
        response.into_response()
    }
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
