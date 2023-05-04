//! Subroutine entity specific [`Repository`][`crate::entity::Repository`]

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use serde::{Deserialize, Serialize};

use crate::entity::{self, repository::Result};
use crate::scene;
use crate::subroutine::image;

use super::{entity::Id, Entity, Status};

/// Repository events for the `subroutine` `Repository`
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Event {
    /// Unknown event type (or one we don't understand)
    Unknown,
    /// entity was inserted into the repo
    Insert {
        /// entity that was added
        subroutine: Entity,
    },
    /// existing entity was updated
    Update {
        /// new entity representation
        subroutine: Entity,
        /// prior entity representation
        orig: Entity,
    },
    /// entity was deleted from the repo
    Delete {
        /// entity that was deleted
        subroutine: Entity,
    },
}

/// `subroutine` specific query arguments
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Query<'a> {
    /// `scene` [`Id`][`scene::entity::Id`]
    pub scene_entity_id: Option<&'a scene::entity::Id>,
    /// `subroutine` image [`Id`][`image::Id`]
    pub image_id: Option<&'a image::Id>,
}

impl<'a> Query<'a> {
    /// Construct a query builder
    #[must_use]
    pub fn builder() -> Self {
        Self::default()
    }

    /// Scope the query to the provided `scene`
    pub fn for_scene_entity(&mut self, id: &'a scene::entity::Id) -> &mut Self {
        self.scene_entity_id = Some(id);
        self
    }

    /// Scope the query to the provided `image`
    pub fn for_image(&mut self, id: &'a image::Id) -> &mut Self {
        self.image_id = Some(id);
        self
    }

    /// Convert this builder instance into a [`Query`]
    #[must_use]
    pub fn build(&self) -> Self {
        Self {
            scene_entity_id: self.scene_entity_id,
            image_id: self.image_id,
        }
    }
}

impl<'a> entity::repository::Query for Query<'a> {
    type Entity = Entity;

    fn matches(&self, record: &Entity) -> bool {
        if self.scene_entity_id.is_none() && self.image_id.is_none() {
            true
        } else {
            if let Some(scene_entity_id) = self.scene_entity_id {
                if scene_entity_id != &record.scene_entity_id {
                    return false;
                }
            }
            if let Some(subroutine_image_id) = self.image_id {
                if subroutine_image_id != &record.image_id {
                    return false;
                }
            }
            true
        }
    }
}

/// Repository methods specific to the `subroutine` [`Entity`]
#[cfg_attr(test, automock)]
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    /// Store the provided [`Entity`]
    async fn subroutines_create(&self, subroutine: Entity) -> Result<Entity>;
    /// Delete an [`Entity`] matching the provided [`Id`]
    async fn subroutines_delete(&self, id: &Id) -> Result<()>;
    /// Determine whether or not entities exist matching the provided [`Query`]
    async fn subroutines_exists<'a>(&self, query: Query<'a>) -> Result<bool>;
    /// Retrieve one or more entities matching the provided [`Query`]
    async fn subroutines_find<'a>(&self, query: Query<'a>) -> Result<Vec<Entity>>;
    /// Retrieve a subroutine entity matching the specified [`Id`]
    async fn subroutines_get(&self, id: &Id) -> Result<Entity>;
    /// Update an existing subroutine [`Entity`]
    async fn subroutines_update(&self, id: &Id, status: Option<Status>) -> Result<Entity>;
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
