use std::sync::Arc;

use axum::extract::{Path, State};

use crate::core::scene::entity::{
    api::State as SceneState,
    service::{delete::Input, Delete},
};
use crate::entity;
use crate::utils::server::http::DeleteResponse;

/// Delete the given scene entity from the server
///
/// # Errors
///
/// - Scene id is invalid (or does not exist)
/// - repository error occurred
pub async fn delete_scene<A, S>(
    State(state): State<Arc<A>>,
    Path(scene): Path<String>,
) -> Result<DeleteResponse, entity::service::Error>
where
    A: SceneState<S>,
    S: Delete,
{
    state
        .scene_entity_service()
        .delete(&Input::new(&scene))
        .await?;

    Ok(DeleteResponse)
}
