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

use crate::config::HolodekkConfig;
use crate::core::projectors::{CreateProjector, DeleteProjector, FindProjectors, GetProjector};
use crate::core::subroutines::{
    self, api::server::SubroutinesApiServices, CreateSubroutine, FindSubroutines,
};
use crate::core::ApiCoreState;

#[cfg_attr(test, automock)]
pub trait ProjectorApiServices<P> {
    fn projectors(&self) -> Arc<P>;
}

pub fn router<A, S, P, C>(services: Arc<A>) -> axum::Router
where
    A: ProjectorApiServices<P>
        + SubroutinesApiServices<S>
        + ApiCoreState<C>
        + Send
        + Sync
        + 'static,
    P: CreateProjector + DeleteProjector + FindProjectors + GetProjector + Send + Sync + 'static,
    S: CreateSubroutine + FindSubroutines + Send + Sync + 'static,
    C: HolodekkConfig,
{
    Router::new()
        .route("/", get(list::handler))
        .route("/", post(create::handler))
        .route("/:projector", delete(stop::handler))
        .nest(
            "/:projector/subroutines",
            subroutines::api::server::router(services.clone()),
        )
        .with_state(services)
}
