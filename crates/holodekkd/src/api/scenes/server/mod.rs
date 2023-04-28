mod create;
mod delete;
mod list;

use std::sync::Arc;

use axum::{
    routing::{delete, get},
    Router,
};

use holodekk::core::repositories::ScenesRepository;

use crate::api::ApiState;

pub fn router<T>(state: Arc<ApiState<T>>) -> axum::Router
where
    T: ScenesRepository + Send + Sync + 'static,
{
    Router::new()
        .route("/", get(list::handler).post(create::handler))
        .route("/:scene", delete(self::delete::handler))
        // .nest(
        //     "/:projector/subroutines",
        //     subroutines::server::router(services.clone()),
        // )
        .with_state(state)
}
