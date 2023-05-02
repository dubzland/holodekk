use std::sync::Arc;

use axum::{
    routing::{delete, get},
    Router,
};

use crate::services::scene::SceneEntityServiceMethods;
use crate::services::subroutine::SubroutineEntityServiceMethods;

use super::ApiState;

pub fn router<A, E, U>(state: Arc<A>) -> Router<Arc<A>>
where
    A: ApiState<E, U>,
    E: SceneEntityServiceMethods,
    U: SubroutineEntityServiceMethods,
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
    mod create_subroutine;
    pub use create_subroutine::*;
    mod delete_subroutine;
    pub use delete_subroutine::*;
    mod find_subroutines;
    pub use find_subroutines::*;
}

pub mod models;
