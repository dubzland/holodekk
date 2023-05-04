use std::sync::Arc;

use axum::{response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

use holodekk::apis::http::ApiState;
use holodekk::scene;
use holodekk::subroutine;

pub struct HolodekkdApiState<R>
where
    R: scene::entity::Repository + subroutine::entity::Repository,
{
    repo: Arc<R>,
    scene_entity_service: Arc<scene::entity::Service<R>>,
    subroutine_entity_service: Arc<subroutine::entity::Service<R>>,
}

impl<R> HolodekkdApiState<R>
where
    R: scene::entity::Repository + subroutine::entity::Repository,
{
    pub fn new(repo: Arc<R>) -> Self {
        let scene_entity_service = Arc::new(scene::entity::Service::new(repo.clone()));
        let subroutine_entity_service = Arc::new(subroutine::entity::Service::new(repo.clone()));
        Self {
            repo,
            scene_entity_service,
            subroutine_entity_service,
        }
    }

    #[must_use]
    pub fn repo(&self) -> Arc<R> {
        self.repo.clone()
    }
}

impl<R> ApiState<scene::entity::Service<R>, subroutine::entity::Service<R>> for HolodekkdApiState<R>
where
    R: scene::entity::Repository + subroutine::entity::Repository,
{
    fn scene_entity_service(&self) -> Arc<scene::entity::Service<R>> {
        self.scene_entity_service.clone()
    }

    fn subroutine_entity_service(&self) -> Arc<subroutine::entity::Service<R>> {
        self.subroutine_entity_service.clone()
    }
}

pub fn router<R>(api_state: Arc<HolodekkdApiState<R>>) -> axum::Router
where
    R: scene::entity::Repository + subroutine::entity::Repository,
{
    Router::new()
        .route("/health", get(health))
        .nest("/scenes", scene::entity::api::router(api_state))
}

#[derive(Debug, Deserialize, Serialize)]
struct HealthResponse {
    status: String,
}

#[allow(clippy::unused_async)]
async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "OK".to_string(),
    })
}
