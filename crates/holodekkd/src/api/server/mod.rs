mod projectors;

use std::sync::Arc;

use axum::{response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

use holodekk::{
    config::HolodekkConfig, core::repositories::ProjectorRepository,
    core::services::projectors::ProjectorsService, managers::projector::ProjectorCommand,
};

pub struct ApiServices<T>
where
    T: ProjectorRepository,
{
    projectors_service: Arc<ProjectorsService<T>>,
}

impl<T> ApiServices<T>
where
    T: ProjectorRepository,
{
    pub fn projectors(&self) -> Arc<ProjectorsService<T>> {
        self.projectors_service.clone()
    }
}

pub fn router<C, T>(config: Arc<C>, repo: Arc<T>, cmd_tx: Sender<ProjectorCommand>) -> axum::Router
where
    C: HolodekkConfig,
    T: ProjectorRepository,
{
    // Create the global services
    let projectors_service = Arc::new(ProjectorsService::new(config, repo, cmd_tx));
    let services = Arc::new(ApiServices { projectors_service });

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

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
