use std::sync::Arc;

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::services::{
    scene::SceneEntityServiceMethods, subroutine::SubroutineEntityServiceMethods,
};

use super::{commands, ApiState};

pub fn subroutines<A, E, U>(state: Arc<A>) -> Router<Arc<A>>
where
    A: ApiState<E, U>,
    E: Send + Sync + 'static,
    U: SubroutineEntityServiceMethods,
{
    Router::new()
        .route("/", post(commands::subroutine::create))
        .with_state(state)
}

pub fn scenes<A, E, U>(state: Arc<A>) -> Router
where
    A: ApiState<E, U>,
    E: SceneEntityServiceMethods,
    U: SubroutineEntityServiceMethods,
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
