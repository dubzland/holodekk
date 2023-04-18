use std::path::PathBuf;
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::core::{
    api::ApiError,
    subroutine_definitions::{
        api::models::NewSubroutineDefinition, CreateSubroutineDefinition,
        SubroutineDefinitionsCreateInput,
    },
};

use super::SubroutineDefinitionsApiServices;

pub async fn handler<S, D>(
    State(state): State<Arc<S>>,
    Json(new_subroutine_definition): Json<NewSubroutineDefinition>,
) -> Result<impl IntoResponse, ApiError>
where
    S: SubroutineDefinitionsApiServices<D>,
    D: CreateSubroutineDefinition,
{
    let definition = state
        .definitions()
        .create(&SubroutineDefinitionsCreateInput::new(
            &new_subroutine_definition.name,
            &PathBuf::from(new_subroutine_definition.path),
            new_subroutine_definition.kind,
        ))
        .await?;
    Ok((StatusCode::CREATED, Json(definition)))
}
