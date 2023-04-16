use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
};

use crate::core::projectors::services::{DeleteProjector, ProjectorsDeleteInput};

use super::{internal_error, ApiServices};

pub async fn handler<P, D>(
    State(state): State<Arc<ApiServices<P, D>>>,
    Path(id): Path<String>,
) -> Result<(), (StatusCode, String)>
where
    P: DeleteProjector,
{
    state
        .projectors()
        .delete(&ProjectorsDeleteInput::new(&id))
        .await
        .map_err(internal_error)?;
    Ok(())
}
