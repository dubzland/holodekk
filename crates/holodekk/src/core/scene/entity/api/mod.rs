//! The HTTP API for `scene` entity management.

use std::sync::Arc;

use axum::{
    routing::{delete, get},
    Router,
};
#[cfg(test)]
use mockall::automock;

use crate::core::scene;

/// State required by the [`scene`] api.
#[cfg_attr(test, automock)]
pub trait State<S>: Send + Sync + 'static
where
    S: Send + Sync + 'static,
{
    /// returns an instance of the `scene` [`Service`][`scene::entity::Service`]
    fn scene_entity_service(&self) -> Arc<S>;
}

/// Axum router for handling scene (and subroutine) requests.
pub fn router<A, S>() -> Router<Arc<A>>
where
    A: State<S>,
    S: scene::entity::service::Methods,
{
    Router::new()
        .route("/", get(commands::find_scenes).post(commands::create_scene))
        .route("/:scene", delete(commands::delete_scene))
    // .with_state(state)
}

pub mod commands {
    //! Api commands (CRUD) for `scene` entities
    mod create_scene;
    pub use create_scene::*;
    mod delete_scene;
    pub use delete_scene::*;
    mod find_scenes;
    pub use find_scenes::*;
}

pub mod models;
