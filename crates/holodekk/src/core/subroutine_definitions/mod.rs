pub mod api;
pub mod entities;
mod init;
pub mod repositories;
pub mod services;

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::config::HolodekkConfig;
use crate::core::services::Result;
use crate::core::subroutine_definitions::entities::{SubroutineDefinition, SubroutineKind};

#[derive(Clone, Debug)]
pub struct SubroutineDefinitionsCreateInput<'c> {
    name: &'c str,
    path: &'c PathBuf,
    kind: SubroutineKind,
}

impl<'c> SubroutineDefinitionsCreateInput<'c> {
    pub fn new(name: &'c str, path: &'c PathBuf, kind: SubroutineKind) -> Self {
        Self { name, path, kind }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn path(&self) -> &PathBuf {
        self.path
    }

    pub fn kind(&self) -> SubroutineKind {
        self.kind
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CreateSubroutineDefinition {
    async fn create<'a>(
        &self,
        input: &'a SubroutineDefinitionsCreateInput<'a>,
    ) -> Result<SubroutineDefinition>;
}

pub async fn create_service<C>(config: Arc<C>) -> Result<services::SubroutineDefinitionsService>
where
    C: HolodekkConfig,
{
    let definitions = init::initialize_subroutine_definitions(config)?;

    Ok(services::SubroutineDefinitionsService::new(
        std::sync::RwLock::new(definitions),
    ))
}
