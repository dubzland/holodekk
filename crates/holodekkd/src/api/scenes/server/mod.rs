mod create;
mod delete;
mod list;

use std::sync::Arc;

use axum::{
    routing::{delete, get, post},
    Router,
};

use holodekk::core::services::ScenesServiceMethods;

pub fn router<A, S>(state: Arc<A>) -> axum::Router
where
    A: ScenesApiState<S>,
    S: ScenesServiceMethods,
{
    Router::new()
        .route("/", post(create::handler))
        // .route("/", get(list::handler).post(create::handler))
        // .route("/:scene", delete(self::delete::handler))
        // .nest(
        //     "/:projector/subroutines",
        //     subroutines::server::router(services.clone()),
        // )
        .with_state(state)
}
