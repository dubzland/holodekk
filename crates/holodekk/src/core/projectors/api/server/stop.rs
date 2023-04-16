use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
};

use crate::core::projectors::services::{DeleteProjector, ProjectorsDeleteInput};

use super::{internal_error, ApiServices};

pub async fn handler<S>(
    State(state): State<Arc<ApiServices<S>>>,
    Path(id): Path<String>,
) -> Result<(), (StatusCode, String)>
where
    S: DeleteProjector,
{
    state
        .projectors()
        .delete(&ProjectorsDeleteInput::new(&id))
        .await
        .map_err(internal_error)?;
    Ok(())
}
