use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use sha2::{Digest, Sha256};

use crate::core::entities::SubroutineInstance;

use super::{RepositoryId, RepositoryQuery, Result};

pub fn subroutine_instance_repo_id(fleet: &str, namespace: &str, subroutine_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(fleet);
    hasher.update(namespace);
    hasher.update(subroutine_id);
    format!("{:x}", hasher.finalize())
}

impl RepositoryId for SubroutineInstance {
    fn id(&self) -> String {
        subroutine_instance_repo_id(&self.fleet, &self.namespace, &self.subroutine_id)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SubroutineInstancesQuery {
    pub fleet: Option<String>,
    pub namespace: Option<String>,
    pub subroutine_id: Option<String>,
}

impl SubroutineInstancesQuery {
    pub fn for_subroutine(&mut self, id: &str) -> &mut Self {
        self.subroutine_id = Some(id.into());
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
}

impl RepositoryQuery for SubroutineInstancesQuery {
    type Entity = SubroutineInstance;

    fn builder() -> Self {
        Self::default()
    }

    fn matches(&self, record: &SubroutineInstance) -> bool {
        if self.fleet.is_none() && self.namespace.is_none() && self.subroutine_id.is_none() {
            true
        } else {
            if let Some(fleet) = self.fleet.as_ref() {
                if fleet != &record.fleet {
                    return false;
                }
            }
            if let Some(namespace) = self.namespace.as_ref() {
                if namespace != &record.namespace {
                    return false;
                }
            }
            if let Some(subroutine_id) = self.subroutine_id.as_ref() {
                if subroutine_id != &record.subroutine_id {
                    return false;
                }
            }
            true
        }
    }

    fn build(&self) -> Self {
        Self {
            fleet: self.fleet.clone(),
            namespace: self.namespace.clone(),
            subroutine_id: self.subroutine_id.clone(),
        }
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait SubroutineInstancesRepository: Send + Sync {
    async fn subroutine_instances_create(
        &self,
        instance: SubroutineInstance,
    ) -> Result<SubroutineInstance>;
    async fn subroutine_instances_delete(&self, id: &str) -> Result<()>;
    async fn subroutine_instances_exists(&self, id: &str) -> Result<bool>;
    async fn subroutine_instances_find<T>(&self, query: T) -> Result<Vec<SubroutineInstance>>
    where
        T: RepositoryQuery<Entity = SubroutineInstance> + 'static;
    async fn subroutine_instances_get(&self, id: &str) -> Result<SubroutineInstance>;
}
