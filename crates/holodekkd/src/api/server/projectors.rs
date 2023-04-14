use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;

use holodekk::core::{
    entities::Projector,
    repositories::ProjectorsRepository,
    services::projectors::{
        Create, Delete, Exists, Find, ProjectorsCreateInput, ProjectorsDeleteInput,
        ProjectorsExistsInput, ProjectorsFindInput,
    },
};

use super::ApiServices;

pub fn routes<T>() -> Router<Arc<ApiServices<T>>>
where
    T: ProjectorsRepository + 'static,
{
    Router::new()
        .route("/", get(list))
        .route("/", post(start))
        .route("/:namespace", delete(stop))
}

async fn list<T>(State(state): State<Arc<ApiServices<T>>>) -> impl IntoResponse
where
    T: ProjectorsRepository,
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

async fn start<T>(
    State(state): State<Arc<ApiServices<T>>>,
    Json(new_projector): Json<NewProjector>,
) -> Result<Json<Projector>, (StatusCode, String)>
where
    T: ProjectorsRepository,
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

async fn stop<T>(
    State(state): State<Arc<ApiServices<T>>>,
    Path(namespace): Path<String>,
) -> Result<(), (StatusCode, String)>
where
    T: ProjectorsRepository,
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
