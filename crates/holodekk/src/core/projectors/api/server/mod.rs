mod list;
mod start;
mod stop;

use std::sync::Arc;

use axum::{
    http::StatusCode,
    routing::{delete, get, post},
    Router,
};

use crate::core::projectors::services::{
    CreateProjector, DeleteProjector, FindProjectors, ProjectorExists,
};

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
    S: CreateProjector + DeleteProjector + FindProjectors + ProjectorExists + Send + Sync + 'static,
{
    // Create the global services
    let services = Arc::new(ApiServices { projectors_service });

    Router::new()
        .route("/", get(list::handler))
        .route("/", post(start::handler))
        .route("/:namespace", delete(stop::handler))
        .with_state(services)
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
