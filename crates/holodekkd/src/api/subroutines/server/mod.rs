mod create;
mod delete;
mod list;

use std::sync::Arc;

use axum::{
    routing::{delete, get},
    Router,
};
#[cfg(test)]
use mockall::{automock, predicate::*};

use holodekk::config::HolodekkConfig;
use holodekk::core::projectors::GetProjector;
use holodekk::core::subroutines::SubroutinesServiceMethods;

use crate::api::{ApiCoreState, ProjectorsApiServices};

#[cfg_attr(test, automock)]
pub trait SubroutinesApiServices<S> {
    fn subroutines(&self) -> Arc<S>;
}

pub fn router<S, A, C, P>(services: Arc<A>) -> axum::Router<Arc<A>>
where
    A: SubroutinesApiServices<S>
        + ProjectorsApiServices<P>
        + ApiCoreState<C>
        + Send
        + Sync
        + 'static,
    S: SubroutinesServiceMethods,
    P: GetProjector + Send + Sync + 'static,
    C: HolodekkConfig,
{
    Router::new()
        .route("/", get(list::handler).post(create::handler))
        .route("/:subroutine", delete(delete::handler))
        .with_state(services)
}
