mod create;

use std::sync::Arc;

use axum::{routing::post, Router};
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::core::subroutine_definitions::CreateSubroutineDefinition;

#[cfg_attr(test, automock)]
pub trait SubroutineDefinitionsApiServices<D>
where
    D: CreateSubroutineDefinition,
{
    fn definitions(&self) -> Arc<D>;
}

pub fn router<S, D>(services: Arc<S>) -> axum::Router
where
    S: SubroutineDefinitionsApiServices<D> + Send + Sync + 'static,
    D: CreateSubroutineDefinition + Send + Sync + 'static,
{
    Router::new()
        // .route("/", get(list::handler))
        .route("/", post(create::handler))
        // .route("/:id", delete(stop::handler))
        .with_state(services)
}
