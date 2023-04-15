mod projectors;

use std::sync::Arc;

use axum::{response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

use crate::core::projectors::{repositories::ProjectorsRepository, services::ProjectorsService};

pub struct ApiServices<R>
where
    R: ProjectorsRepository,
{
    projectors_service: Arc<ProjectorsService<R>>,
}

impl<R> ApiServices<R>
where
    R: ProjectorsRepository,
{
    pub fn projectors(&self) -> Arc<ProjectorsService<R>> {
        self.projectors_service.clone()
    }
}

pub fn router<R>(projectors_service: Arc<ProjectorsService<R>>) -> axum::Router
where
    R: ProjectorsRepository + 'static,
{
    // Create the global services
    let services = Arc::new(ApiServices { projectors_service });

    Router::new()
        .route("/health", get(health))
        .merge(projectors::routes())
        .with_state(services)
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
