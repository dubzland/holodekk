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

pub struct ApiServices<S> {
    projectors_service: Arc<S>,
}

impl<S> ApiServices<S> {
    pub fn projectors(&self) -> Arc<S> {
        self.projectors_service.clone()
    }
}

pub fn router<S>(projectors_service: Arc<S>) -> axum::Router
where
    S: CreateProjector + DeleteProjector + FindProjectors + GetProjector + Send + Sync + 'static,
{
    // Create the global services
    let services = Arc::new(ApiServices { projectors_service });

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
