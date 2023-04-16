use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
};

use crate::core::projectors::services::{DeleteProjector, ProjectorsDeleteInput};

use super::{internal_error, ProjectorApiServices};

pub async fn handler<S, P>(
    State(state): State<Arc<S>>,
    Path(id): Path<String>,
) -> Result<(), (StatusCode, String)>
where
    S: ProjectorApiServices<P>,
    P: DeleteProjector,
{
    state
        .projectors()
        .delete(&ProjectorsDeleteInput::new(&id))
        .await
        .map_err(internal_error)?;
    Ok(())
}
