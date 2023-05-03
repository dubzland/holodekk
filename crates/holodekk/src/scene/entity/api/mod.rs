use std::sync::Arc;

use axum::{
    routing::{delete, get},
    Router,
};

use crate::apis::http::ApiState;
use crate::scene;
use crate::subroutine;

pub fn router<A, E, U>(state: Arc<A>) -> Router
where
    A: ApiState<E, U>,
    E: scene::entity::service::Methods,
    U: subroutine::entity::service::Methods,
{
    Router::new()
        .route("/", get(commands::find_scenes).post(commands::create_scene))
        .route("/:scene", delete(commands::delete_scene))
        .nest(
            "/:scene/subroutines",
            subroutine::entity::api::router(state.clone()),
        )
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

pub mod models;
