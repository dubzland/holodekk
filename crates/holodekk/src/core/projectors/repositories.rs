use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::repositories::{RepositoryQuery, Result};

use super::entities::ProjectorEntity;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectorsQuery {
    namespace: Option<String>,
}

impl ProjectorsQuery {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn namespace_eq(&mut self, namespace: &str) -> &mut Self {
        self.namespace = Some(namespace.to_string());
        self
    }

    pub fn build(&self) -> Self {
        Self {
            namespace: self.namespace.clone(),
        }
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }
}

impl RepositoryQuery for ProjectorsQuery {
    type Entity = ProjectorEntity;

    fn matches(&self, projector: &ProjectorEntity) -> bool {
        if let Some(namespace) = self.namespace.as_ref() {
            projector.namespace() == namespace
        } else {
            true
        }
    }
}

impl PartialEq<ProjectorsQuery> for ProjectorEntity {
    fn eq(&self, other: &ProjectorsQuery) -> bool {
        other.matches(self)
    }
}

impl PartialEq<ProjectorEntity> for ProjectorsQuery {
    fn eq(&self, other: &ProjectorEntity) -> bool {
        self.matches(other)
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ProjectorsRepository: Send + Sync {
    async fn projectors_create(&self, projector: ProjectorEntity) -> Result<ProjectorEntity>;
    async fn projectors_delete(&self, id: &str) -> Result<()>;
    async fn projectors_exists(&self, query: ProjectorsQuery) -> Result<bool>;
    async fn projectors_find(&self, query: ProjectorsQuery) -> Result<Vec<ProjectorEntity>>;
    async fn projectors_get(&self, id: &str) -> Result<ProjectorEntity>;
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
