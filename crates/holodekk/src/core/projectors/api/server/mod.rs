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

use crate::core::projectors::{CreateProjector, DeleteProjector, FindProjectors, GetProjector};
use crate::core::services::Error;
use crate::core::ApiCoreState;

#[cfg_attr(test, automock)]
pub trait ProjectorApiServices<P> {
    fn projectors(&self) -> Arc<P>;
}

pub fn router<S, P, C>(services: Arc<S>) -> axum::Router
where
    S: ProjectorApiServices<P> + ApiCoreState<C> + Send + Sync + 'static,
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
