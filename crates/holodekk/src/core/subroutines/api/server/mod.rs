mod create;
mod list;

use std::sync::Arc;

use axum::{routing::get, Router};
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::config::HolodekkConfig;
use crate::core::projectors::{api::server::ProjectorsApiServices, GetProjector};
use crate::core::subroutines::SubroutinesServiceMethods;
use crate::core::ApiCoreState;

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
        .with_state(services)
}
