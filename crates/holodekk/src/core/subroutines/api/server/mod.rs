mod create;

use std::sync::Arc;

use axum::{routing::post, Router};
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::config::HolodekkConfig;
use crate::core::projectors::{api::server::ProjectorApiServices, GetProjector};
use crate::core::subroutines::CreateSubroutine;
use crate::core::ApiCoreState;

#[cfg_attr(test, automock)]
pub trait SubroutinesApiServices<S> {
    fn subroutines(&self) -> Arc<S>;
}

pub fn router<S, A, C, P>(services: Arc<A>) -> axum::Router<Arc<A>>
where
    A: SubroutinesApiServices<S>
        + ProjectorApiServices<P>
        + ApiCoreState<C>
        + Send
        + Sync
        + 'static,
    S: CreateSubroutine + Send + Sync + 'static,
    P: GetProjector + Send + Sync + 'static,
    C: HolodekkConfig,
{
    Router::new()
        .route("/", post(create::handler))
        .with_state(services)
}
