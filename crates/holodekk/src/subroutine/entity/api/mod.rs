//! The HTTP API for `subroutine` entity management.

use std::sync::Arc;

use axum::{
    routing::{delete, get},
    Router,
};

use crate::apis::http::ApiState;
use crate::scene::entity::service::Methods as SceneMethods;
use crate::subroutine::entity::service::Methods as SubroutineMethods;

/// Axum router for handling subroutine requests.
pub fn router<A, E, U>(state: Arc<A>) -> Router<Arc<A>>
where
    A: ApiState<E, U>,
    E: SceneMethods,
    U: SubroutineMethods,
{
    Router::new()
        .route(
            "/",
            get(commands::find_subroutines).post(commands::create_subroutine),
        )
        .route("/:subroutine", delete(commands::delete_subroutine))
        .with_state(state)
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
