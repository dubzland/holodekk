//! HTTP specific api elements

use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use log::error;
#[cfg(test)]
use mockall::automock;
use serde::Serialize;

use crate::entity;

/// Global shared state for all http api endpoints
#[cfg_attr(test, automock)]
pub trait ApiState<S1, S2>: Send + Sync + 'static
where
    S1: Send + Sync + 'static,
    S2: Send + Sync + 'static,
{
    /// returns an instance of the `scene` [`Service`][`crate::scene::entity::Service`]
    fn scene_entity_service(&self) -> Arc<S1>;
    /// returns an instance of the `subroutine` [`Service`][`crate::subroutine::entity::Service`]
    fn subroutine_entity_service(&self) -> Arc<S2>;
}

/// Ensures create actions return a 201 - Created code
pub struct CreateResponse<T>(pub T);
impl<T> IntoResponse for CreateResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        (StatusCode::CREATED, Json(self.0)).into_response()
    }
}

/// Ensures delete responses return a 204 - No Content code
pub struct DeleteResponse;
impl IntoResponse for DeleteResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

/// Ensures get responses return a 200 - OK code
pub struct GetResponse<T>(pub T);
impl<T> IntoResponse for GetResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

impl IntoResponse for entity::service::Error {
    fn into_response(self) -> Response {
        match self {
            entity::service::Error::NotUnique(_) => (StatusCode::CONFLICT, self.to_string()),
            entity::service::Error::NotFound(_)
            | entity::service::Error::InvalidEntityId(_)
            | entity::service::Error::InvalidImageId(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            entity::service::Error::Repository(err) => {
                error!("Repository error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
            entity::service::Error::Unexpected(err) => {
                error!("Unexpected error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
        }
        .into_response()
    }
}
