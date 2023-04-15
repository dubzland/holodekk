use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};

use crate::core::projectors::{
    api::models::NewProjector,
    entities::Projector,
    services::{
        Create, Delete, Exists, Find, ProjectorsCreateInput, ProjectorsDeleteInput,
        ProjectorsExistsInput, ProjectorsFindInput,
    },
};

pub struct ApiServices<S>
where
    S: Send + Sync + 'static,
{
    projectors_service: Arc<S>,
}

impl<S> ApiServices<S>
where
    S: Send + Sync + 'static,
{
    pub fn projectors(&self) -> Arc<S> {
        self.projectors_service.clone()
    }
}

pub fn router<S>(projectors_service: Arc<S>) -> axum::Router
where
    S: Create + Delete + Exists + Find + Send + Sync + 'static,
{
    // Create the global services
    let services = Arc::new(ApiServices { projectors_service });

    Router::new()
        .route("/", get(list))
        .route("/", post(start))
        .route("/:namespace", delete(stop))
        .with_state(services)
}

async fn list<S>(
    State(state): State<Arc<ApiServices<S>>>,
) -> Result<Json<Vec<Projector>>, (StatusCode, String)>
where
    S: Find + Send + Sync,
{
    let projectors = state
        .projectors()
        .find(ProjectorsFindInput::default())
        .await
        .unwrap();
    Ok(Json(projectors))
}

async fn start<S>(
    State(state): State<Arc<ApiServices<S>>>,
    Json(new_projector): Json<NewProjector>,
) -> Result<Json<Projector>, (StatusCode, String)>
where
    S: Create + Exists + Send + Sync,
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

async fn stop<S>(
    State(state): State<Arc<ApiServices<S>>>,
    Path(namespace): Path<String>,
) -> Result<(), (StatusCode, String)>
where
    S: Delete + Exists + Send + Sync,
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
