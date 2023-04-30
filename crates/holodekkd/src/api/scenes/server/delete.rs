use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use holodekk::core::{repositories::ScenesRepository, scene_delete};

use crate::api::{ApiError, ApiState};

pub async fn handler<T>(
    State(state): State<Arc<ApiState<T>>>,
    Path(scene): Path<String>,
) -> Result<impl IntoResponse, ApiError>
where
    T: ScenesRepository,
{
    let id = scene.try_into()?;

    scene_delete::execute(state.repo(), scene_delete::Request { id: &id }).await?;
    Ok(())
}
