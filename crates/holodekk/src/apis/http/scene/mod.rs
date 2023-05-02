use std::sync::Arc;

use axum::{
    routing::{delete, get},
    Router,
};

use crate::services::{
    scene::SceneEntityServiceMethods, subroutine::SubroutineEntityServiceMethods,
};

use super::{subroutine, ApiState};

pub fn router<A, E, U>(state: Arc<A>) -> Router
where
    A: ApiState<E, U>,
    E: SceneEntityServiceMethods,
    U: SubroutineEntityServiceMethods,
{
    Router::new()
        .route("/", get(commands::find_scenes).post(commands::create_scene))
        .route("/:scene", delete(commands::delete_scene))
        .nest("/:scene/subroutines", subroutine::router(state.clone()))
        .with_state(state)
}

pub mod commands {
    mod create_scene;
    pub use create_scene::*;
    mod delete_scene;
    pub use delete_scene::*;
    mod find_scenes;
    pub use find_scenes::*;
}
