use std::sync::Arc;

use axum::{response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

use holodekk::apis::http::{routers, ApiState};
use holodekk::core::{
    entities::{SceneEntityRepository, SubroutineEntityRepository},
    services::{scene::SceneEntityService, subroutine::SubroutinesService},
};
use holodekk::utils::{
    servers::{start_http_server, HttpServerHandle},
    ConnectionInfo,
};

pub struct HolodekkdApiState<R>
where
    R: SceneEntityRepository + SubroutineEntityRepository,
{
    repo: Arc<R>,
    scene_entity_service: Arc<SceneEntityService<R>>,
    subroutines_service: Arc<SubroutinesService<R>>,
}

impl<R> HolodekkdApiState<R>
where
    R: SceneEntityRepository + SubroutineEntityRepository,
{
    pub fn new(repo: Arc<R>) -> Self {
        let scene_entity_service = Arc::new(SceneEntityService::new(repo.clone()));
        let subroutines_service = Arc::new(SubroutinesService::new(repo.clone()));
        Self {
            repo,
            scene_entity_service,
            subroutines_service,
        }
    }

    pub fn repo(&self) -> Arc<R> {
        self.repo.clone()
    }
}

impl<R> ApiState<SceneEntityService<R>, SubroutinesService<R>> for HolodekkdApiState<R>
where
    R: SceneEntityRepository + SubroutineEntityRepository,
{
    fn scene_entity_service(&self) -> Arc<SceneEntityService<R>> {
        self.scene_entity_service.clone()
    }

    fn subroutines_service(&self) -> Arc<SubroutinesService<R>> {
        self.subroutines_service.clone()
    }
}

pub fn router<R>(api_state: Arc<HolodekkdApiState<R>>) -> axum::Router
where
    R: SceneEntityRepository + SubroutineEntityRepository,
{
    Router::new()
        .route("/health", get(health))
        .nest("/scenes", routers::scenes(api_state))
}

pub struct Server {
    handle: HttpServerHandle,
}

impl Server {
    pub fn new(handle: HttpServerHandle) -> Self {
        Self { handle }
    }

    pub fn start<R>(config: &ConnectionInfo, repo: Arc<R>) -> Self
    where
        R: SceneEntityRepository + SubroutineEntityRepository,
    {
        let state = HolodekkdApiState::new(repo);
        let handle = start_http_server(config, router(Arc::new(state)));

        Self::new(handle)
    }

    pub async fn stop(&mut self) {
        self.handle.stop().await.unwrap();
    }
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
