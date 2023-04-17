use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use crate::core::projectors::services::{DeleteProjector, ProjectorsDeleteInput};

use super::ProjectorApiServices;

pub async fn handler<S, P>(
    State(state): State<Arc<S>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, crate::core::services::Error>
where
    S: ProjectorApiServices<P>,
    P: DeleteProjector,
{
    state
        .projectors()
        .delete(&ProjectorsDeleteInput::new(&id))
        .await?;
    Ok(())
}
