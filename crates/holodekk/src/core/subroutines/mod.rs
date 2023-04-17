pub mod api;
pub mod entities;
pub mod repositories;
pub mod services;
pub mod worker;

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::core::services::Result;

use entities::Subroutine;

#[derive(Clone, Debug)]
pub struct SubroutinesCreateInput {
    pub fleet: String,
    pub namespace: String,
    pub subroutine_definition_id: String,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CreateSubroutine {
    async fn create(&self, input: SubroutinesCreateInput) -> Result<Subroutine>;
}
