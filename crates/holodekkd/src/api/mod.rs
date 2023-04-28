mod errors;
pub use errors::ApiError;
mod scenes;
// #[cfg(test)]
// mod fixtures;
// mod subroutine_definitions;
// mod subroutines;

use std::sync::Arc;

use axum::{response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

use holodekk::core::repositories::ScenesRepository;

// use holodekk::scenes::SceneMethods;

pub struct ApiState<T>
where
    T: ScenesRepository,
{
    repo: Arc<T>,
}

impl<T> ApiState<T>
where
    T: ScenesRepository,
{
    pub fn new(repo: Arc<T>) -> Self {
        Self { repo }
    }

    pub fn repo(&self) -> Arc<T> {
        self.repo.clone()
    }
}

pub fn router<T>(api_state: Arc<ApiState<T>>) -> axum::Router
where
    T: ScenesRepository + 'static,
{
    Router::new()
        .route("/health", get(health))
        .nest("/scenes", scenes::server::router(api_state.clone()))
    // .nest(
    //     "/subroutine_definitions",
    //     subroutine_definitions::server::router(api_services),
    // )
}

#[derive(Debug, Deserialize, Serialize)]
struct HealthResponse {
    status: String,
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "OK".to_string(),
    })
}
