#[cfg(test)]
mod fixtures;
mod projectors;
mod subroutine_definitions;
mod subroutines;

use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use log::error;
use serde::{Deserialize, Serialize};

use holodekk::config::HolodekkConfig;
use holodekk::core::projectors::{ProjectorsError, ProjectorsServiceMethods};
use holodekk::core::subroutine_definitions::{
    SubroutineDefinitionsError, SubroutineDefinitionsServiceMethods,
};
use holodekk::core::subroutines::{SubroutinesError, SubroutinesServiceMethods};

use projectors::server::ProjectorsApiServices;
use subroutine_definitions::server::SubroutineDefinitionsApiServices;
use subroutines::server::SubroutinesApiServices;

pub trait ApiCoreState<C> {
    fn config(&self) -> Arc<C>;
}

pub struct ApiState<P, D, S, C>
where
    P: ProjectorsServiceMethods,
    D: SubroutineDefinitionsServiceMethods,
    S: SubroutinesServiceMethods,
    C: HolodekkConfig,
{
    projectors_service: Arc<P>,
    subroutines_service: Arc<S>,
    definitions_service: Arc<D>,
    config: Arc<C>,
}

impl<P, D, S, C> ApiState<P, D, S, C>
where
    P: ProjectorsServiceMethods,
    D: SubroutineDefinitionsServiceMethods,
    S: SubroutinesServiceMethods,
    C: HolodekkConfig,
{
    pub fn new(
        projectors_service: Arc<P>,
        definitions_service: Arc<D>,
        subroutines_service: Arc<S>,
        config: Arc<C>,
    ) -> Self {
        Self {
            projectors_service,
            definitions_service,
            subroutines_service,
            config,
        }
    }
}

impl<P, D, S, C> ApiCoreState<C> for ApiState<P, D, S, C>
where
    P: ProjectorsServiceMethods,
    D: SubroutineDefinitionsServiceMethods,
    S: SubroutinesServiceMethods,
    C: HolodekkConfig,
{
    fn config(&self) -> Arc<C> {
        self.config.clone()
    }
}

impl<P, D, S, C> ProjectorsApiServices<P> for ApiState<P, D, S, C>
where
    P: ProjectorsServiceMethods,
    D: SubroutineDefinitionsServiceMethods,
    S: SubroutinesServiceMethods,
    C: HolodekkConfig,
{
    fn projectors(&self) -> Arc<P> {
        self.projectors_service.clone()
    }
}

impl<P, D, S, C> SubroutineDefinitionsApiServices<D> for ApiState<P, D, S, C>
where
    P: ProjectorsServiceMethods,
    D: SubroutineDefinitionsServiceMethods,
    S: SubroutinesServiceMethods,
    C: HolodekkConfig,
{
    fn definitions(&self) -> Arc<D> {
        self.definitions_service.clone()
    }
}

impl<P, D, S, C> SubroutinesApiServices<S> for ApiState<P, D, S, C>
where
    P: ProjectorsServiceMethods,
    D: SubroutineDefinitionsServiceMethods,
    S: SubroutinesServiceMethods,
    C: HolodekkConfig,
{
    fn subroutines(&self) -> Arc<S> {
        self.subroutines_service.clone()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Unexpected projector error occurred")]
    Projector(#[from] ProjectorsError),
    #[error("Unexpected subroutine definition error occurred")]
    SubroutineDefinition(#[from] SubroutineDefinitionsError),
    #[error("Unexpected subroutine error occurred")]
    Subroutine(#[from] SubroutinesError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let response = match self {
            ApiError::Projector(err) => {
                error!("Error encountered: {:?}", err);
                match err {
                    ProjectorsError::AlreadyRunning(id) => (
                        StatusCode::CONFLICT,
                        format!("Projector already running with id {}", id),
                    ),
                    ProjectorsError::NotFound(id) => (
                        StatusCode::NOT_FOUND,
                        format!("Could not find a projector with id {}", id),
                    ),
                    err => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unexpected projector error occurred: {}", err),
                    ),
                }
            }
            ApiError::Subroutine(err) => {
                error!("Error encountered: {:?}", err);
                match err {
                    SubroutinesError::AlreadyRunning => (
                        StatusCode::CONFLICT,
                        "Subroutine already running".to_string(),
                    ),
                    SubroutinesError::NotFound(id) => (
                        StatusCode::NOT_FOUND,
                        format!("Could not find a subroutine with id {}", id),
                    ),
                    err => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unexpected subroutine error occurred: {}", err),
                    ),
                }
            }
            ApiError::SubroutineDefinition(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unexpected subroutine definition error occurred: {}", err),
            ),
        };
        response.into_response()
    }
}

pub fn router<P, D, S, C>(api_services: Arc<ApiState<P, D, S, C>>) -> axum::Router
where
    P: ProjectorsServiceMethods,
    D: SubroutineDefinitionsServiceMethods,
    S: SubroutinesServiceMethods,
    C: HolodekkConfig,
{
    Router::new()
        .route("/health", get(health))
        .nest(
            "/projectors",
            projectors::server::router(api_services.clone()),
        )
        .nest(
            "/subroutine_definitions",
            subroutine_definitions::server::router(api_services),
        )
}

#[derive(Debug, Deserialize, Serialize)]
struct HealthResponse {
    status: String,
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "OK".to_string(),
    })
}
