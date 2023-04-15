use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;

use holodekk::core::projectors::{
    entities::Projector,
    repositories::ProjectorsRepository,
    services::{
        Create, Delete, Exists, Find, ProjectorsCreateInput, ProjectorsDeleteInput,
        ProjectorsExistsInput, ProjectorsFindInput,
    },
};

use super::ApiServices;

pub fn routes<R>() -> Router<Arc<ApiServices<R>>>
where
    R: ProjectorsRepository + 'static,
{
    Router::new()
        .route("/", get(list))
        .route("/", post(start))
        .route("/:namespace", delete(stop))
}

async fn list<R>(State(state): State<Arc<ApiServices<R>>>) -> impl IntoResponse
where
    R: ProjectorsRepository,
{
    let projectors = state
        .projectors()
        .find(ProjectorsFindInput::default())
        .await
        .unwrap();
    Json(projectors)
}

#[derive(Deserialize)]
struct NewProjector {
    namespace: String,
}

async fn start<R>(
    State(state): State<Arc<ApiServices<R>>>,
    Json(new_projector): Json<NewProjector>,
) -> Result<Json<Projector>, (StatusCode, String)>
where
    R: ProjectorsRepository,
{
    if state
        .projectors()
        .exists(ProjectorsExistsInput {
            namespace: new_projector.namespace.clone(),
        })
        .await
        .map_err(internal_error)?
    {
        Err((
            StatusCode::CONFLICT,
            "Projector already running for specified namespace".to_string(),
        ))
    } else {
        let projector = state
            .projectors()
            .create(ProjectorsCreateInput {
                namespace: new_projector.namespace,
            })
            .await
            .map_err(internal_error)?;
        Ok(Json(projector))
    }
}

async fn stop<R>(
    State(state): State<Arc<ApiServices<R>>>,
    Path(namespace): Path<String>,
) -> Result<(), (StatusCode, String)>
where
    R: ProjectorsRepository,
{
    if state
        .projectors()
        .exists(ProjectorsExistsInput {
            namespace: namespace.clone(),
        })
        .await
        .map_err(internal_error)?
    {
        state
            .projectors()
            .delete(ProjectorsDeleteInput { namespace })
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
