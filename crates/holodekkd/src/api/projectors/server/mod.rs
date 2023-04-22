mod create;
mod list;
mod stop;

use std::sync::Arc;

use axum::{
    routing::{delete, get, post},
    Router,
};
#[cfg(test)]
use mockall::{automock, predicate::*};

use holodekk::config::HolodekkConfig;
use holodekk::core::projectors::ProjectorsServiceMethods;
use holodekk::core::subroutines::SubroutinesServiceMethods;

use crate::api::subroutines::{self, server::SubroutinesApiServices};
use crate::api::ApiCoreState;

#[cfg_attr(test, automock)]
pub trait ProjectorsApiServices<P> {
    fn projectors(&self) -> Arc<P>;
}

pub fn router<A, P, S, C>(services: Arc<A>) -> axum::Router
where
    A: ProjectorsApiServices<P>
        + SubroutinesApiServices<S>
        + ApiCoreState<C>
        + Send
        + Sync
        + 'static,
    P: ProjectorsServiceMethods,
    S: SubroutinesServiceMethods,
    C: HolodekkConfig,
{
    Router::new()
        .route("/", get(list::handler))
        .route("/", post(create::handler))
        .route("/:projector", delete(stop::handler))
        .nest(
            "/:projector/subroutines",
            subroutines::server::router(services.clone()),
        )
        .with_state(services)
}
