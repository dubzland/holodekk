use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use serde::{Deserialize, Serialize};

use crate::entity::{self, repository::Result};
use crate::scene;
use crate::subroutine::image;

use super::{entity::Id, Entity, Status};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Event {
    Unknown,
    Insert { subroutine: Entity },
    Update { subroutine: Entity, orig: Entity },
    Delete { subroutine: Entity },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Query<'a> {
    pub scene_entity_id: Option<&'a scene::entity::Id>,
    pub image_id: Option<&'a image::Id>,
}

impl<'a> Query<'a> {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn for_scene_entity(&mut self, id: &'a scene::entity::Id) -> &mut Self {
        self.scene_entity_id = Some(id);
        self
    }

    pub fn for_image(&mut self, id: &'a image::Id) -> &mut Self {
        self.image_id = Some(id);
        self
    }

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

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    async fn subroutines_create(&self, subroutine: Entity) -> Result<Entity>;
    async fn subroutines_delete(&self, id: &Id) -> Result<()>;
    async fn subroutines_exists<'a>(&self, query: Query<'a>) -> Result<bool>;
    async fn subroutines_find<'a>(&self, query: Query<'a>) -> Result<Vec<Entity>>;
    async fn subroutines_get(&self, id: &Id) -> Result<Entity>;
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
