use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::apis::http::ApiState;
use crate::core::services::scene::{DeleteScene, Error, ScenesDeleteInput};

pub async fn delete<A, S>(
    State(state): State<Arc<A>>,
    Path(scene): Path<String>,
) -> Result<impl IntoResponse, Error>
where
    A: ApiState<S>,
    S: DeleteScene,
{
    state
        .scenes_service()
        .delete(&ScenesDeleteInput::new(&scene))
        .await?;
    Ok((StatusCode::NO_CONTENT, Json(())))
}
