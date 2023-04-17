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
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::core::projectors::services::{
    CreateProjector, DeleteProjector, FindProjectors, GetProjector,
};
use crate::core::services::Error;

#[cfg_attr(test, automock)]
pub trait ProjectorApiServices<P> {
    fn projectors(&self) -> Arc<P>;
}

pub fn router<S, P>(services: Arc<S>) -> axum::Router
where
    S: ProjectorApiServices<P> + Send + Sync + 'static,
    P: CreateProjector + DeleteProjector + FindProjectors + GetProjector + Send + Sync + 'static,
{
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
