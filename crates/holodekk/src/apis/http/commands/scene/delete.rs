use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::apis::http::ApiState;
use crate::core::services::{
    scene::{DeleteScene, ScenesDeleteInput},
    Error,
};

pub async fn delete<A, E, U>(
    State(state): State<Arc<A>>,
    Path(scene): Path<String>,
) -> Result<impl IntoResponse, Error>
where
    A: ApiState<E, U>,
    E: DeleteScene,
    U: Send + Sync + 'static,
{
    state
        .scenes_service()
        .delete(&ScenesDeleteInput::new(&scene))
        .await?;
    Ok((StatusCode::NO_CONTENT, Json(())))
}
