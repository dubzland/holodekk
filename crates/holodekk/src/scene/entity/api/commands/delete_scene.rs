use std::sync::Arc;

use axum::extract::{Path, State};

use crate::apis::http::{ApiState, DeleteResponse};
use crate::entity;
use crate::scene::entity::service::{delete::Input, Delete};

pub async fn delete_scene<A, E, U>(
    State(state): State<Arc<A>>,
    Path(scene): Path<String>,
) -> Result<DeleteResponse, entity::service::Error>
where
    A: ApiState<E, U>,
    E: Delete,
    U: Send + Sync + 'static,
{
    state
        .scene_entity_service()
        .delete(&Input::new(&scene))
        .await?;

    Ok(DeleteResponse)
}