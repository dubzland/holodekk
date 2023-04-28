use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::core::entities::{
    SceneEntityId, SubroutineDefinitionEntityId, SubroutineEntity, SubroutineEntityId,
};
use crate::core::enums::SubroutineStatus;
use crate::core::subroutines_find;

use crate::repositories::{RepositoryQuery, Result};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SubroutinesQuery<'a> {
    pub scene_id: Option<&'a SceneEntityId>,
    pub subroutine_definition_id: Option<&'a SubroutineDefinitionEntityId>,
}

impl<'a> From<subroutines_find::Request<'a>> for SubroutinesQuery<'a> {
    fn from(request: subroutines_find::Request<'a>) -> SubroutinesQuery<'a> {
        Self {
            scene_id: request.scene_id,
            subroutine_definition_id: request.subroutine_definition_id,
        }
    }
}

impl<'a> SubroutinesQuery<'a> {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn for_scene(&mut self, id: &'a SceneEntityId) -> &mut Self {
        self.scene_id = Some(id);
        self
    }

    pub fn for_subroutine_definition(&mut self, id: &'a SubroutineDefinitionEntityId) -> &mut Self {
        self.subroutine_definition_id = Some(id);
        self
    }

    pub fn build(&self) -> Self {
        Self {
            scene_id: self.scene_id,
            subroutine_definition_id: self.subroutine_definition_id,
        }
    }
}

impl<'a> RepositoryQuery for SubroutinesQuery<'a> {
    type Entity = SubroutineEntity;

    fn matches(&self, record: &SubroutineEntity) -> bool {
        if self.scene_id.is_none() && self.subroutine_definition_id.is_none() {
            true
        } else {
            if let Some(scene_id) = self.scene_id {
                if scene_id != &record.scene_id {
                    return false;
                }
            }
            if let Some(subroutine_definition_id) = self.subroutine_definition_id {
                if subroutine_definition_id != &record.subroutine_definition_id {
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
