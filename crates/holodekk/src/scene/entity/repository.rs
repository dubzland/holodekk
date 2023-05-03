use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use serde::{Deserialize, Serialize};

use super::{Entity, Id, Name, Status};
use crate::entity::{self, repository::Query as RepositoryQuery};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Event {
    Unknown,
    Insert { scene: Entity },
    Update { scene: Entity, orig: Entity },
    Delete { scene: Entity },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Query<'a> {
    name: Option<&'a str>,
    status: Option<&'a Status>,
}

impl<'a> Query<'a> {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn name_eq(&mut self, name: &'a str) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn status_eq(&mut self, status: &'a Status) -> &mut Self {
        self.status = Some(status);
        self
    }

    #[must_use]
    pub fn build(&self) -> Self {
        Self {
            name: self.name,
            status: self.status,
        }
    }

    pub fn name(&self) -> Option<&'a str> {
        self.name
    }

    pub fn status(&self) -> Option<&Status> {
        self.status
    }
}

impl<'a> From<&'a Entity> for Query<'a> {
    fn from(scene: &'a Entity) -> Self {
        Self::builder()
            .name_eq(&scene.name)
            .status_eq(&scene.status)
            .build()
    }
}

impl<'a> crate::entity::repository::Query for Query<'a> {
    type Entity = Entity;

    fn matches(&self, scene: &Entity) -> bool {
        if let Some(name) = self.name {
            &scene.name == name
        } else {
            true
        }
    }
}

impl PartialEq<Query<'_>> for Entity {
    fn eq(&self, other: &Query) -> bool {
        other.matches(self)
    }
}

impl<'a> PartialEq<Entity> for Query<'a> {
    fn eq(&self, other: &Entity) -> bool {
        self.matches(other)
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    async fn scenes_create(&self, scene: Entity) -> entity::repository::Result<Entity>;
    async fn scenes_delete(&self, id: &Id) -> entity::repository::Result<()>;
    async fn scenes_exists<'a>(&self, query: Query<'a>) -> entity::repository::Result<bool>;
    async fn scenes_find<'a>(&self, query: Query<'a>) -> entity::repository::Result<Vec<Entity>>;
    async fn scenes_get(&self, id: &Id) -> entity::repository::Result<Entity>;
    async fn scenes_update(
        &self,
        id: &Id,
        name: Option<Name>,
        status: Option<Status>,
    ) -> entity::repository::Result<Entity>;
}

#[cfg(test)]
mod fixtures {
    use rstest::fixture;

    #[fixture]
    pub fn mock_repository() -> super::MockRepository {
        super::MockRepository::default()
    }
}

#[cfg(test)]
pub use fixtures::*;
