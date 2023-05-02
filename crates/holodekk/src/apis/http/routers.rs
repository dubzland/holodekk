use std::sync::Arc;

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::core::services::{scene::ScenesServiceMethods, subroutine::SubroutinesServiceMethods};

use super::{commands, ApiState};

pub fn subroutines<A, E, U>(state: Arc<A>) -> Router<Arc<A>>
where
    A: ApiState<E, U>,
    E: Send + Sync + 'static,
    U: SubroutinesServiceMethods,
{
    Router::new()
        .route("/", post(commands::subroutine::create))
        .with_state(state)
}

pub fn scenes<A, E, U>(state: Arc<A>) -> Router
where
    A: ApiState<E, U>,
    E: ScenesServiceMethods,
    U: SubroutinesServiceMethods,
{
    Router::new()
        .route(
            "/",
            get(commands::scene::find).post(commands::scene::create),
        )
        .route("/:scene", delete(commands::scene::delete))
        .nest("/:scene/subroutines", subroutines(state.clone()))
        .with_state(state)
}
