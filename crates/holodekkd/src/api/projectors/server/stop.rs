use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use holodekk::core::projectors::{DeleteProjector, ProjectorsDeleteInput};

use crate::api::ApiError;

use super::ProjectorsApiServices;

pub async fn handler<S, P>(
    State(state): State<Arc<S>>,
    Path(projector): Path<String>,
) -> Result<impl IntoResponse, ApiError>
where
    S: ProjectorsApiServices<P>,
    P: DeleteProjector,
{
    state
        .projectors()
        .delete(&ProjectorsDeleteInput::new(&projector))
        .await?;
    Ok(())
}
