//! Scene entity specific [`Repository`][`crate::entity::Repository`]

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use serde::{Deserialize, Serialize};

use crate::entity::repository::{Query as RepositoryQuery, Result};

use super::{Entity, Id, Name, Status};

/// Repository events for the `Scene` `Repository`
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Event {
    /// Unknown event type (or one we don't understand)
    Unknown,
    /// entity was inserted into the repo
    Insert {
        /// entity that was added
        scene: Entity,
    },
    /// existing entity was updated
    Update {
        /// new entity representation
        scene: Entity,
        /// prior entity representation
        orig: Entity,
    },
    /// entity was deleted from the repo
    Delete {
        /// entity that was deleted
        scene: Entity,
    },
}

/// `scene` specific query arguments
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Query<'a> {
    name: Option<&'a str>,
    status: Option<&'a Status>,
}

impl<'a> Query<'a> {
    /// Construct a query builder
    #[must_use]
    pub fn builder() -> Self {
        Self::default()
    }

    /// match on the spefied name
    pub fn name_eq(&mut self, name: &'a str) -> &mut Self {
        self.name = Some(name);
        self
    }

    /// match on the spefied status
    pub fn status_eq(&mut self, status: &'a Status) -> &mut Self {
        self.status = Some(status);
        self
    }

    /// Convert this builder instance into a [`Query`]
    #[must_use]
    pub fn build(&self) -> Self {
        Self {
            name: self.name,
            status: self.status,
        }
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

impl<'a> RepositoryQuery for Query<'a> {
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

/// Repository methods specific to the `scene` [`Entity`]
#[cfg_attr(test, automock)]
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    /// Store the provided [`Entity`]
    async fn scenes_create(&self, scene: Entity) -> Result<Entity>;
    /// Delete an [`Entity`] matching the provided [`Id`]
    async fn scenes_delete(&self, id: &Id) -> Result<()>;
    /// Determine whether or not entities exist matching the provided [`Query`]
    async fn scenes_exists<'a>(&self, query: Query<'a>) -> Result<bool>;
    /// Retrieve one or more entities matching the provided [`Query`]
    async fn scenes_find<'a>(&self, query: Query<'a>) -> Result<Vec<Entity>>;
    /// Retrieve a scene entity matching the specified [`Id`]
    async fn scenes_get(&self, id: &Id) -> Result<Entity>;
    /// Update an existing scene [`Entity`]
    async fn scenes_update(
        &self,
        id: &Id,
        name: Option<Name>,
        status: Option<Status>,
    ) -> Result<Entity>;
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
