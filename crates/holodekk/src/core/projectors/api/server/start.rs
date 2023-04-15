use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};

use crate::core::projectors::{
    api::models::NewProjector,
    entities::Projector,
    services::{CreateProjector, ProjectorExists, ProjectorsCreateInput, ProjectorsExistsInput},
};

use super::{internal_error, ApiServices};

pub async fn handler<S>(
    State(state): State<Arc<ApiServices<S>>>,
    Json(new_projector): Json<NewProjector>,
) -> Result<Json<Projector>, (StatusCode, String)>
where
    S: CreateProjector + ProjectorExists,
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
