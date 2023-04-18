use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use crate::core::api::ApiError;
use crate::core::projectors::{DeleteProjector, ProjectorsDeleteInput};

use super::ProjectorApiServices;

pub async fn handler<S, P>(
    State(state): State<Arc<S>>,
    Path(projector): Path<String>,
) -> Result<impl IntoResponse, ApiError>
where
    S: ProjectorApiServices<P>,
    P: DeleteProjector,
{
    state
        .projectors()
        .delete(&ProjectorsDeleteInput::new(&projector))
        .await?;
    Ok(())
}
