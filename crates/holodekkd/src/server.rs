use std::sync::Arc;

use axum::{response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

use holodekk::apis::http::{routers, ApiState};
use holodekk::core::{repositories::ScenesRepository, services::scene::ScenesService};

pub struct HolodekkdApiState<R>
where
    R: ScenesRepository,
{
    repo: Arc<R>,
    scenes_service: Arc<ScenesService<R>>,
}

impl<R> HolodekkdApiState<R>
where
    R: ScenesRepository,
{
    pub fn new(repo: Arc<R>) -> Self {
        let scenes_service = Arc::new(ScenesService::new(repo.clone()));
        Self {
            repo,
            scenes_service,
        }
    }

    pub fn repo(&self) -> Arc<R> {
        self.repo.clone()
    }
}

impl<R> ApiState<ScenesService<R>> for HolodekkdApiState<R>
where
    R: ScenesRepository,
{
    fn scenes_service(&self) -> Arc<ScenesService<R>> {
        self.scenes_service.clone()
    }
}

pub fn router<R>(api_state: Arc<HolodekkdApiState<R>>) -> axum::Router
where
    R: ScenesRepository,
{
    Router::new()
        .route("/health", get(health))
        .nest("/scenes", routers::scenes(api_state))
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
