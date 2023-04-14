use std::path::PathBuf;

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use sha2::{Digest, Sha256};

use crate::core::entities::{Subroutine, SubroutineKind};

use super::{RepositoryId, RepositoryQuery, Result};

pub fn subroutine_repo_id(name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(name);
    format!("{:x}", hasher.finalize())
}

impl RepositoryId for Subroutine {
    fn id(&self) -> String {
        subroutine_repo_id(&self.name)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SubroutinesQuery {
    name: Option<String>,
    path: Option<PathBuf>,
    kind: Option<SubroutineKind>,
}

impl SubroutinesQuery {
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

impl RepositoryQuery for SubroutinesQuery {
    type Entity = Subroutine;

    fn builder() -> Self {
        Self::default()
    }
    fn matches(&self, record: &Subroutine) -> bool {
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

impl PartialEq<SubroutinesQuery> for Subroutine {
    fn eq(&self, other: &SubroutinesQuery) -> bool {
        other.matches(self)
    }
}

impl PartialEq<Subroutine> for SubroutinesQuery {
    fn eq(&self, other: &Subroutine) -> bool {
        self.matches(other)
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait SubroutinesRepository: Send + Sync + 'static {
    async fn subroutines_create(&self, subroutine: Subroutine) -> Result<Subroutine>;
    async fn subroutines_delete(&self, id: &str) -> Result<()>;
    async fn subroutines_exists(&self, id: &str) -> Result<bool>;
    async fn subroutines_find<T>(&self, query: T) -> Result<Vec<Subroutine>>
    where
        T: RepositoryQuery<Entity = Subroutine> + 'static;
    async fn subroutines_get(&self, id: &str) -> Result<Subroutine>;
}
