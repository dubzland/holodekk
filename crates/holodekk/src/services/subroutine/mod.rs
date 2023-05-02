use std::sync::Arc;

use crate::entities::{SubroutineEntity, SubroutineEntityRepository};

use super::EntityServiceResult;

use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CreateSubroutine: Send + Sync + 'static {
    async fn create<'c>(
        &self,
        input: &'c CreateSubroutineInput<'c>,
    ) -> EntityServiceResult<SubroutineEntity>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait DeleteSubroutine: Send + Sync + 'static {
    async fn delete<'c>(&self, input: &'c DeleteSubroutineInput<'c>) -> EntityServiceResult<()>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait FindSubroutines: Send + Sync + 'static {
    async fn find<'a>(
        &self,
        input: &'a FindSubroutinesInput<'a>,
    ) -> EntityServiceResult<Vec<SubroutineEntity>>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetSubroutine: Send + Sync + 'static {
    async fn get<'c>(
        &self,
        input: &'c GetSubroutineInput<'c>,
    ) -> EntityServiceResult<SubroutineEntity>;
}

#[derive(Clone, Debug)]
pub struct CreateSubroutineInput<'c> {
    pub scene_entity_id: &'c str,
    pub subroutine_image_id: &'c str,
}

impl<'c> CreateSubroutineInput<'c> {
    pub fn new(scene_entity_id: &'c str, subroutine_image_id: &'c str) -> Self {
        Self {
            scene_entity_id,
            subroutine_image_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DeleteSubroutineInput<'c> {
    pub id: &'c str,
}

impl<'c> DeleteSubroutineInput<'c> {
    pub fn new(id: &'c str) -> Self {
        Self { id }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FindSubroutinesInput<'f> {
    pub scene_entity_id: Option<&'f str>,
    pub subroutine_image_id: Option<&'f str>,
}

impl<'f> FindSubroutinesInput<'f> {
    pub fn new(scene_entity_id: Option<&'f str>, subroutine_image_id: Option<&'f str>) -> Self {
        Self {
            scene_entity_id,
            subroutine_image_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GetSubroutineInput<'c> {
    pub id: &'c str,
}

impl<'c> GetSubroutineInput<'c> {
    pub fn new(id: &'c str) -> Self {
        Self { id }
    }
}

pub trait SubroutineEntityServiceMethods:
    CreateSubroutine + DeleteSubroutine + FindSubroutines + GetSubroutine
{
}
impl<T> SubroutineEntityServiceMethods for T where
    T: CreateSubroutine + DeleteSubroutine + FindSubroutines + GetSubroutine
{
}

#[derive(Debug)]
pub struct SubroutineEntityService<R>
where
    R: SubroutineEntityRepository,
{
    repo: Arc<R>,
}

impl<R> SubroutineEntityService<R>
where
    R: SubroutineEntityRepository,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

mod create;
mod delete;
mod find;
mod get;

#[cfg(test)]
pub mod fixtures {
    use mockall::mock;
    use rstest::*;

    use super::*;

    mock! {
        pub SubroutineEntityService {}
        #[async_trait]
        impl CreateSubroutine for SubroutineEntityService {
            async fn create<'a>(&self, input: &'a CreateSubroutineInput<'a>) -> EntityServiceResult<SubroutineEntity>;
        }

        #[async_trait]
        impl DeleteSubroutine for SubroutineEntityService {
            async fn delete<'a>(&self, input: &'a DeleteSubroutineInput<'a>) -> EntityServiceResult<()>;
        }

        #[async_trait]
        impl FindSubroutines for SubroutineEntityService {
            async fn find<'a>(&self, input: &'a FindSubroutinesInput<'a>) -> EntityServiceResult<Vec<SubroutineEntity>>;
        }

        #[async_trait]
        impl GetSubroutine for SubroutineEntityService {
            async fn get<'a>(&self, input: &'a GetSubroutineInput<'a>) -> EntityServiceResult<SubroutineEntity>;
        }
    }

    #[fixture]
    pub fn mock_create_subroutine() -> MockCreateSubroutine {
        MockCreateSubroutine::default()
    }

    #[fixture]
    pub fn mock_delete_subroutine() -> MockDeleteSubroutine {
        MockDeleteSubroutine::default()
    }

    #[fixture]
    pub fn mock_find_subroutines() -> MockFindSubroutines {
        MockFindSubroutines::default()
    }

    #[fixture]
    pub fn mock_get_subroutine() -> MockGetSubroutine {
        MockGetSubroutine::default()
    }

    #[fixture]
    pub fn mock_subroutine_service() -> MockSubroutineEntityService {
        MockSubroutineEntityService::default()
    }
}
