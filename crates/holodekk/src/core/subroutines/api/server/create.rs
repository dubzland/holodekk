use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::config::HolodekkConfig;
use crate::core::projectors::{
    api::server::ProjectorsApiServices, GetProjector, ProjectorsGetInput,
};
use crate::core::subroutines::{
    api::models::NewSubroutine, CreateSubroutine, SubroutinesCreateInput,
};
use crate::core::ApiCoreState;

use super::SubroutinesApiServices;

pub async fn handler<A, S, P, C>(
    State(state): State<Arc<A>>,
    Path(projector): Path<String>,
    Json(new_subroutine): Json<NewSubroutine>,
) -> Result<impl IntoResponse, crate::core::api::ApiError>
where
    A: SubroutinesApiServices<S> + ProjectorsApiServices<P> + ApiCoreState<C>,
    S: CreateSubroutine,
    P: GetProjector,
    C: HolodekkConfig,
{
    let projector = state
        .projectors()
        .get(&ProjectorsGetInput::new(&projector))
        .await?;

    let subroutine = state
        .subroutines()
        .create(&SubroutinesCreateInput::new(
            projector.fleet(),
            projector.namespace(),
            &new_subroutine.subroutine_definition_id,
        ))
        .await?;
    Ok((StatusCode::CREATED, Json(subroutine)))
}
