use std::path::PathBuf;

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use sha2::{Digest, Sha256};

use crate::core::subroutines::entities::{SubroutineDefinition, SubroutineKind};

use crate::core::repositories::{RepositoryId, RepositoryQuery, Result};

pub fn subroutine_definition_repo_id(name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(name);
    format!("{:x}", hasher.finalize())
}

impl RepositoryId for SubroutineDefinition {
    fn id(&self) -> String {
        subroutine_definition_repo_id(&self.name)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SubroutineDefinitionsQuery {
    name: Option<String>,
    path: Option<PathBuf>,
    kind: Option<SubroutineKind>,
}

impl SubroutineDefinitionsQuery {
    pub fn name_eq<S>(&mut self, name: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.name = Some(name.into());
        self
    }

    pub fn path_eq<P>(&mut self, path: P) -> &mut Self
    where
        P: Into<PathBuf>,
    {
        self.path = Some(path.into());
        self
    }

    pub fn kind_eq(&mut self, kind: SubroutineKind) -> &mut Self {
        self.kind = Some(kind);
        self
    }
}

impl RepositoryQuery for SubroutineDefinitionsQuery {
    type Entity = SubroutineDefinition;

    fn builder() -> Self {
        Self::default()
    }
    fn matches(&self, record: &SubroutineDefinition) -> bool {
        if self.name.is_none() && self.path.is_none() && self.kind.is_none() {
            true
        } else {
            if let Some(name) = self.name.as_ref() {
                if name != &record.name {
                    return false;
                }
            }
            if let Some(path) = self.path.as_ref() {
                if path != &record.path {
                    return false;
                }
            }
            if let Some(kind) = self.kind.as_ref() {
                if kind != &record.kind {
                    return false;
                }
            }
            true
        }
    }

    fn build(&self) -> Self {
        Self {
            name: self.name.clone(),
            path: self.path.clone(),
            kind: self.kind,
        }
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait SubroutineDefinitionsRepository: Send + Sync + 'static {
    async fn subroutine_definitions_create(
        &self,
        subroutine: SubroutineDefinition,
    ) -> Result<SubroutineDefinition>;
    async fn subroutine_definitions_delete(&self, id: &str) -> Result<()>;
    async fn subroutine_definitions_exists(&self, id: &str) -> Result<bool>;
    async fn subroutine_definitions_find<T>(&self, query: T) -> Result<Vec<SubroutineDefinition>>
    where
        T: RepositoryQuery<Entity = SubroutineDefinition> + 'static;
    async fn subroutine_definitions_get(&self, id: &str) -> Result<SubroutineDefinition>;
}
