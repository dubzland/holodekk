//! The HTTP API for `subroutine` entity management.

use std::sync::Arc;

use axum::{
    routing::{delete, get},
    Router,
};
#[cfg(test)]
use mockall::automock;

// use crate::apis::http::ApiState;
use crate::core::scene;
use crate::core::subroutine;

/// State required by the [`subroutine`] api.
#[cfg_attr(test, automock)]
pub trait State<S>: Send + Sync + 'static
where
    S: Send + Sync + 'static,
{
    /// returns an instance of the `subroutine`
    /// [`Service`][`crate::core::subroutine::entity::Service`]
    fn subroutine_entity_service(&self) -> Arc<S>;
}

/// Axum router for handling subroutine requests.
pub fn router<A, U, E>() -> Router<Arc<A>>
where
    A: State<U> + scene::entity::api::State<E>,
    U: subroutine::entity::service::Methods,
    E: scene::entity::service::Methods,
{
    Router::new()
        .route(
            "/",
            get(commands::find_subroutines).post(commands::create_subroutine),
        )
        .route("/:subroutine", delete(commands::delete_subroutine))
}

pub mod commands {
    //! Api commands (CRUD) for `subroutine` entities
    mod create_subroutine;
    pub use create_subroutine::*;
    mod delete_subroutine;
    pub use delete_subroutine::*;
    mod find_subroutines;
    pub use find_subroutines::*;
}

pub mod models;
