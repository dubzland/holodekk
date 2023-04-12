use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use holodekk::core::entities::Projector;
use holodekk::core::repositories::ProjectorRepository;
use holodekk::core::services::projectors::{
    All, Exists, ProjectorExistsInput, ProjectorStartInput, ProjectorStopInput, Start, Stop,
};

use super::ApiServices;

pub fn routes<T>() -> Router<Arc<ApiServices<T>>>
where
    T: ProjectorRepository,
{
    Router::new()
        .route("/", get(list))
        .route("/start", post(start))
        .route("/:namespace/stop", post(stop))
}

async fn list<T>(State(state): State<Arc<ApiServices<T>>>) -> impl IntoResponse
where
    T: ProjectorRepository,
{
    let projectors = state.projectors().all().await.unwrap();
    Json(projectors)
}

#[derive(Deserialize)]
struct NewProjector {
    namespace: String,
}

async fn start<T>(
    State(state): State<Arc<ApiServices<T>>>,
    Json(new_projector): Json<NewProjector>,
) -> Result<Json<Projector>, (StatusCode, String)>
where
    T: ProjectorRepository,
{
    if state
        .projectors()
        .exists(ProjectorExistsInput {
            namespace: new_projector.namespace.clone(),
        })
        .await
    {
        Err((
            StatusCode::CONFLICT,
            "Projector already running for specified namespace".to_string(),
        ))
    } else {
        let projector = state
            .projectors()
            .start(ProjectorStartInput {
                namespace: new_projector.namespace,
            })
            .await
            .map_err(internal_error)?;
        Ok(Json(projector))
    }
}

async fn stop<T>(
    State(state): State<Arc<ApiServices<T>>>,
    Path(namespace): Path<String>,
) -> Result<(), (StatusCode, String)>
where
    T: ProjectorRepository,
{
    if state
        .projectors()
        .exists(ProjectorExistsInput {
            namespace: namespace.clone(),
        })
        .await
    {
        state
            .projectors()
            .stop(ProjectorStopInput { namespace })
            .await
            .map_err(internal_error)?;
        Ok(())
    } else {
        Ok(())
    }
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
