use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::core::{
    actions::subroutines_find,
    entities::{SceneEntityId, SubroutineEntity, SubroutineEntityId},
    enums::SubroutineStatus,
    images::SubroutineImageId,
    repositories::{RepositoryQuery, Result},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SubroutinesQuery<'a> {
    pub scene_entity_id: Option<&'a SceneEntityId>,
    pub subroutine_image_id: Option<&'a SubroutineImageId>,
}

impl<'a> From<subroutines_find::Request<'a>> for SubroutinesQuery<'a> {
    fn from(request: subroutines_find::Request<'a>) -> SubroutinesQuery<'a> {
        Self {
            scene_entity_id: request.scene_entity_id,
            subroutine_image_id: request.subroutine_image_id,
        }
    }
}

impl<'a> SubroutinesQuery<'a> {
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

impl<'a> RepositoryQuery for SubroutinesQuery<'a> {
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
pub trait SubroutinesRepository: Send + Sync {
    async fn subroutines_create(&self, subroutine: SubroutineEntity) -> Result<SubroutineEntity>;
    async fn subroutines_delete(&self, id: &SubroutineEntityId) -> Result<()>;
    async fn subroutines_exists<'a>(&self, query: SubroutinesQuery<'a>) -> Result<bool>;
    async fn subroutines_find<'a>(
        &self,
        query: SubroutinesQuery<'a>,
    ) -> Result<Vec<SubroutineEntity>>;
    async fn subroutines_get(&self, id: &SubroutineEntityId) -> Result<SubroutineEntity>;
    async fn subroutines_update(
        &self,
        id: &SubroutineEntityId,
        status: Option<SubroutineStatus>,
    ) -> Result<SubroutineEntity>;
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
