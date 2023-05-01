use std::sync::Arc;

use axum::{
    routing::{delete, get},
    Router,
};

use crate::core::services::scene::ScenesServiceMethods;

use super::{commands, ApiState};

pub fn scenes<A, S>(state: Arc<A>) -> Router
where
    A: ApiState<S>,
    S: ScenesServiceMethods,
{
    Router::new()
        .route(
            "/",
            get(commands::scene::find).post(commands::scene::create),
        )
        .route("/:scene", delete(commands::scene::delete))
        .with_state(state)
}
