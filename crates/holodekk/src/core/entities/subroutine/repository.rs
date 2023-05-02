use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use serde::{Deserialize, Serialize};

use crate::core::{
    entities::repository::{EntityRepositoryQuery, EntityRepositoryResult},
    enums::SubroutineStatus,
    images::SubroutineImageId,
};

use super::{SceneEntityId, SubroutineEntity, SubroutineEntityId};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum SubroutineEntityRepositoryEvent {
    Unknown,
    Insert {
        subroutine: SubroutineEntity,
    },
    Update {
        subroutine: SubroutineEntity,
        orig: SubroutineEntity,
    },
    Delete {
        subroutine: SubroutineEntity,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SubroutineEntityRepositoryQuery<'a> {
    pub scene_entity_id: Option<&'a SceneEntityId>,
    pub subroutine_image_id: Option<&'a SubroutineImageId>,
}

impl<'a> SubroutineEntityRepositoryQuery<'a> {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn for_scene_entity(&mut self, id: &'a SceneEntityId) -> &mut Self {
        self.scene_entity_id = Some(id);
        self
    }

    pub fn for_subroutine_image(&mut self, id: &'a SubroutineImageId) -> &mut Self {
        self.subroutine_image_id = Some(id);
        self
    }

    pub fn build(&self) -> Self {
        Self {
            scene_entity_id: self.scene_entity_id,
            subroutine_image_id: self.subroutine_image_id,
        }
    }
}

impl<'a> EntityRepositoryQuery for SubroutineEntityRepositoryQuery<'a> {
    type Entity = SubroutineEntity;

    fn matches(&self, record: &SubroutineEntity) -> bool {
        if self.scene_entity_id.is_none() && self.subroutine_image_id.is_none() {
            true
        } else {
            if let Some(scene_entity_id) = self.scene_entity_id {
                if scene_entity_id != &record.scene_entity_id {
                    return false;
                }
            }
            if let Some(subroutine_image_id) = self.subroutine_image_id {
                if subroutine_image_id != &record.subroutine_image_id {
                    return false;
                }
            }
            true
        }
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait SubroutineEntityRepository: Send + Sync + 'static {
    async fn subroutines_create(
        &self,
        subroutine: SubroutineEntity,
    ) -> EntityRepositoryResult<SubroutineEntity>;
    async fn subroutines_delete(&self, id: &SubroutineEntityId) -> EntityRepositoryResult<()>;
    async fn subroutines_exists<'a>(
        &self,
        query: SubroutineEntityRepositoryQuery<'a>,
    ) -> EntityRepositoryResult<bool>;
    async fn subroutines_find<'a>(
        &self,
        query: SubroutineEntityRepositoryQuery<'a>,
    ) -> EntityRepositoryResult<Vec<SubroutineEntity>>;
    async fn subroutines_get(
        &self,
        id: &SubroutineEntityId,
    ) -> EntityRepositoryResult<SubroutineEntity>;
    async fn subroutines_update(
        &self,
        id: &SubroutineEntityId,
        status: Option<SubroutineStatus>,
    ) -> EntityRepositoryResult<SubroutineEntity>;
}
