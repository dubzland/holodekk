use std::path::PathBuf;

use sha2::{Digest, Sha256};

use crate::repositories::{RepositoryId, RepositoryQuery};

use super::entities::{SubroutineDefinitionEntity, SubroutineKind};

pub fn subroutine_definition_repo_id(name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(name);
    format!("{:x}", hasher.finalize())
}

impl RepositoryId for SubroutineDefinitionEntity {
    fn id(&self) -> String {
        subroutine_definition_repo_id(self.name())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SubroutineDefinitionsQuery {
    name: Option<String>,
    path: Option<PathBuf>,
    kind: Option<SubroutineKind>,
}

impl SubroutineDefinitionsQuery {
    pub fn builder() -> Self {
        Self::default()
    }

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

    pub fn build(&self) -> Self {
        Self {
            name: self.name.clone(),
            path: self.path.clone(),
            kind: self.kind,
        }
    }
}

impl RepositoryQuery for SubroutineDefinitionsQuery {
    type Entity = SubroutineDefinitionEntity;
    fn matches(&self, record: &SubroutineDefinitionEntity) -> bool {
        if self.name.is_none() && self.path.is_none() && self.kind.is_none() {
            true
        } else {
            if let Some(name) = self.name.as_ref() {
                if name != record.name() {
                    return false;
                }
            }
            if let Some(path) = self.path.as_ref() {
                if path != record.path() {
                    return false;
                }
            }
            if let Some(kind) = self.kind {
                if kind != record.kind() {
                    return false;
                }
            }
            true
        }
    }
}
