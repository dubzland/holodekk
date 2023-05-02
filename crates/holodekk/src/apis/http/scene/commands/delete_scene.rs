use std::sync::Arc;

use axum::extract::{Path, State};

use crate::apis::http::{ApiState, DeleteResponse};
use crate::services::{
    scene::{DeleteScene, DeleteSceneInput},
    EntityServiceError,
};

pub async fn delete_scene<A, E, U>(
    State(state): State<Arc<A>>,
    Path(scene): Path<String>,
) -> Result<DeleteResponse, EntityServiceError>
where
    A: ApiState<E, U>,
    E: DeleteScene,
    U: Send + Sync + 'static,
{
    state
        .scene_entity_service()
        .delete(&DeleteSceneInput::new(&scene))
        .await?;

    Ok(DeleteResponse)
}
