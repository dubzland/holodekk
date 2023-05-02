use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::apis::http::ApiState;
use crate::services::{
    scene::{DeleteScene, DeleteSceneInput},
    EntityServiceError,
};

pub async fn delete<A, E, U>(
    State(state): State<Arc<A>>,
    Path(scene): Path<String>,
) -> Result<impl IntoResponse, EntityServiceError>
where
    A: ApiState<E, U>,
    E: DeleteScene,
    U: Send + Sync + 'static,
{
    state
        .scene_entity_service()
        .delete(&DeleteSceneInput::new(&scene))
        .await?;
    Ok((StatusCode::NO_CONTENT, Json(())))
}
