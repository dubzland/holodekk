use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use sha2::{Digest, Sha256};

use crate::core::repositories::{RepositoryId, RepositoryQuery, Result};

use super::entities::Projector;

pub fn projector_repo_id(fleet: &str, namespace: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(fleet);
    hasher.update(namespace);
    format!("{:x}", hasher.finalize())
}

impl RepositoryId for Projector {
    fn id(&self) -> String {
        projector_repo_id(&self.fleet, &self.namespace)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectorsQuery {
    pub fleet: Option<String>,
}

impl ProjectorsQuery {
    pub fn fleet_eq(&mut self, fleet: &str) -> &mut Self {
        self.fleet = Some(fleet.to_string());
        self
    }
}

impl RepositoryQuery for ProjectorsQuery {
    type Entity = Projector;

    fn builder() -> Self {
        Self::default()
    }

    fn matches(&self, projector: &Projector) -> bool {
        if let Some(fleet) = self.fleet.as_ref() {
            &projector.fleet == fleet
        } else {
            true
        }
    }

    fn build(&self) -> Self {
        Self {
            fleet: self.fleet.clone(),
        }
    }
}

impl PartialEq<ProjectorsQuery> for Projector {
    fn eq(&self, other: &ProjectorsQuery) -> bool {
        other.matches(self)
    }
}

impl PartialEq<Projector> for ProjectorsQuery {
    fn eq(&self, other: &Projector) -> bool {
        self.matches(other)
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ProjectorsRepository: Send + Sync {
    async fn projectors_create(&self, projector: Projector) -> Result<Projector>;
    async fn projectors_delete(&self, id: &str) -> Result<()>;
    async fn projectors_exists(&self, id: &str) -> Result<bool>;
    async fn projectors_find<T>(&self, query: T) -> Result<Vec<Projector>>
    where
        T: RepositoryQuery<Entity = Projector> + 'static;
    async fn projectors_get(&self, id: &str) -> Result<Projector>;
}

#[cfg(test)]
pub(crate) mod fixtures {
    use rstest::*;

    use super::MockProjectorsRepository;

    #[fixture]
    pub(crate) fn projectors_repository() -> MockProjectorsRepository {
        MockProjectorsRepository::default()
    }
}
