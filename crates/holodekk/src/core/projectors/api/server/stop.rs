use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
};

use crate::core::projectors::services::{
    DeleteProjector, ProjectorExists, ProjectorsDeleteInput, ProjectorsExistsInput,
};

use super::{internal_error, ApiServices};

pub async fn handler<S>(
    State(state): State<Arc<ApiServices<S>>>,
    Path(namespace): Path<String>,
) -> Result<(), (StatusCode, String)>
where
    S: ProjectorExists + DeleteProjector,
{
    if state
        .projectors()
        .exists(ProjectorsExistsInput {
            namespace: namespace.clone(),
        })
        .await
        .map_err(internal_error)?
    {
        state
            .projectors()
            .delete(ProjectorsDeleteInput { namespace })
            .await
            .map_err(internal_error)?;
        Ok(())
    } else {
        Ok(())
    }
}
