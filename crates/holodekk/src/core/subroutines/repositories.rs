use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use sha2::{Digest, Sha256};

use crate::core::subroutines::entities::Subroutine;

use crate::core::repositories::{RepositoryId, RepositoryQuery, Result};

pub fn subroutine_repo_id(fleet: &str, namespace: &str, subroutine_definition_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(fleet);
    hasher.update(namespace);
    hasher.update(subroutine_definition_id);
    format!("{:x}", hasher.finalize())
}

impl RepositoryId for Subroutine {
    fn id(&self) -> String {
        subroutine_repo_id(
            self.fleet(),
            self.namespace(),
            self.subroutine_definition_id(),
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SubroutinesQuery {
    pub fleet: Option<String>,
    pub namespace: Option<String>,
    pub subroutine_definition_id: Option<String>,
}

impl SubroutinesQuery {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn for_subroutine_definition(&mut self, id: &str) -> &mut Self {
        self.subroutine_definition_id = Some(id.into());
        self
    }

    pub fn fleet_eq(&mut self, fleet: &str) -> &mut Self {
        self.fleet = Some(fleet.into());
        self
    }

    pub fn namespace_eq(&mut self, namespace: &str) -> &mut Self {
        self.namespace = Some(namespace.into());
        self
    }

    pub fn build(&self) -> Self {
        Self {
            fleet: self.fleet.clone(),
            namespace: self.namespace.clone(),
            subroutine_definition_id: self.subroutine_definition_id.clone(),
        }
    }
}

impl RepositoryQuery for SubroutinesQuery {
    type Entity = Subroutine;

    fn matches(&self, record: &Subroutine) -> bool {
        if self.fleet.is_none()
            && self.namespace.is_none()
            && self.subroutine_definition_id.is_none()
        {
            true
        } else {
            if let Some(fleet) = self.fleet.as_ref() {
                if fleet != record.fleet() {
                    return false;
                }
            }
            if let Some(namespace) = self.namespace.as_ref() {
                if namespace != record.namespace() {
                    return false;
                }
            }
            if let Some(subroutine_definition_id) = self.subroutine_definition_id.as_ref() {
                if subroutine_definition_id != record.subroutine_definition_id() {
                    return false;
                }
            }
            true
        }
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait SubroutinesRepository: Send + Sync {
    async fn subroutines_create(&self, instance: Subroutine) -> Result<Subroutine>;
    async fn subroutines_delete(&self, id: &str) -> Result<()>;
    async fn subroutines_exists(&self, id: &str) -> Result<bool>;
    async fn subroutines_find<T>(&self, query: T) -> Vec<Subroutine>
    where
        T: RepositoryQuery<Entity = Subroutine> + 'static;
    async fn subroutines_get(&self, id: &str) -> Result<Subroutine>;
}

#[cfg(test)]
pub(crate) mod fixtures {
    use rstest::*;

    use super::MockSubroutinesRepository;

    #[fixture]
    pub(crate) fn subroutines_repository() -> MockSubroutinesRepository {
        MockSubroutinesRepository::default()
    }
}
