use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::core::subroutines::entities::SubroutineEntity;

use crate::repositories::{RepositoryQuery, Result};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SubroutinesQuery<'a> {
    projector_id: Option<&'a str>,
    subroutine_definition_id: Option<&'a str>,
}

impl<'a> SubroutinesQuery<'a> {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn for_subroutine_definition(&mut self, id: &'a str) -> &mut Self {
        self.subroutine_definition_id = Some(id);
        self
    }

    pub fn for_projector(&mut self, id: &'a str) -> &mut Self {
        self.projector_id = Some(id);
        self
    }

    pub fn build(&self) -> Self {
        Self {
            projector_id: self.projector_id,
            subroutine_definition_id: self.subroutine_definition_id,
        }
    }

    pub fn projector_id(&self) -> Option<&str> {
        self.projector_id
    }

    pub fn subroutine_definition_id(&self) -> Option<&str> {
        self.subroutine_definition_id
    }
}

impl<'a> RepositoryQuery for SubroutinesQuery<'a> {
    type Entity = SubroutineEntity;

    fn matches(&self, record: &SubroutineEntity) -> bool {
        if self.projector_id.is_none() && self.subroutine_definition_id.is_none() {
            true
        } else {
            if let Some(projector_id) = self.projector_id {
                if projector_id != record.projector_id() {
                    return false;
                }
            }
            if let Some(subroutine_definition_id) = self.subroutine_definition_id {
                if subroutine_definition_id != record.subroutine_definition_id() {
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
    async fn subroutines_create(&self, instance: SubroutineEntity) -> Result<SubroutineEntity>;
    async fn subroutines_delete(&self, id: &str) -> Result<()>;
    async fn subroutines_exists<'a>(&self, query: &'a SubroutinesQuery<'a>) -> Result<bool>;
    async fn subroutines_find<'a>(
        &self,
        query: &'a SubroutinesQuery<'a>,
    ) -> Result<Vec<SubroutineEntity>>;
    async fn subroutines_get(&self, id: &str) -> Result<SubroutineEntity>;
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
