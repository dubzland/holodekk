use std::sync::Arc;

use crate::core::entities::{SubroutineEntity, SubroutineEntityRepository};

use super::Result;

use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CreateSubroutine: Send + Sync + 'static {
    async fn create<'c>(&self, input: &'c SubroutinesCreateInput<'c>) -> Result<SubroutineEntity>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait DeleteSubroutine: Send + Sync + 'static {
    async fn delete<'c>(&self, input: &'c SubroutinesDeleteInput<'c>) -> Result<()>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait FindSubroutines: Send + Sync + 'static {
    async fn find<'a>(&self, input: &'a SubroutinesFindInput<'a>) -> Result<Vec<SubroutineEntity>>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetSubroutine: Send + Sync + 'static {
    async fn get<'c>(&self, input: &'c SubroutinesGetInput<'c>) -> Result<SubroutineEntity>;
}

#[derive(Clone, Debug)]
pub struct SubroutinesCreateInput<'c> {
    pub scene_entity_id: &'c str,
    pub subroutine_image_id: &'c str,
}

impl<'c> SubroutinesCreateInput<'c> {
    pub fn new(scene_entity_id: &'c str, subroutine_image_id: &'c str) -> Self {
        Self {
            scene_entity_id,
            subroutine_image_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubroutinesDeleteInput<'c> {
    pub id: &'c str,
}

impl<'c> SubroutinesDeleteInput<'c> {
    pub fn new(id: &'c str) -> Self {
        Self { id }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubroutinesFindInput<'f> {
    pub scene_entity_id: Option<&'f str>,
    pub subroutine_image_id: Option<&'f str>,
}

impl<'f> SubroutinesFindInput<'f> {
    pub fn new(scene_entity_id: Option<&'f str>, subroutine_image_id: Option<&'f str>) -> Self {
        Self {
            scene_entity_id,
            subroutine_image_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubroutinesGetInput<'c> {
    pub id: &'c str,
}

impl<'c> SubroutinesGetInput<'c> {
    pub fn new(id: &'c str) -> Self {
        Self { id }
    }
}

pub trait SubroutinesServiceMethods:
    CreateSubroutine + DeleteSubroutine + FindSubroutines + GetSubroutine
{
}
impl<T> SubroutinesServiceMethods for T where
    T: CreateSubroutine + DeleteSubroutine + FindSubroutines + GetSubroutine
{
}

#[derive(Debug)]
pub struct SubroutinesService<R>
where
    R: SubroutineEntityRepository,
{
    repo: Arc<R>,
}

impl<R> SubroutinesService<R>
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
        pub SubroutinesService {}
        #[async_trait]
        impl CreateSubroutine for SubroutinesService {
            async fn create<'a>(&self, input: &'a SubroutinesCreateInput<'a>) -> Result<SubroutineEntity>;
        }

        #[async_trait]
        impl DeleteSubroutine for SubroutinesService {
            async fn delete<'a>(&self, input: &'a SubroutinesDeleteInput<'a>) -> Result<()>;
        }

        #[async_trait]
        impl FindSubroutines for SubroutinesService {
            async fn find<'a>(&self, input: &'a SubroutinesFindInput<'a>) -> Result<Vec<SubroutineEntity>>;
        }

        #[async_trait]
        impl GetSubroutine for SubroutinesService {
            async fn get<'a>(&self, input: &'a SubroutinesGetInput<'a>) -> Result<SubroutineEntity>;
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
    pub fn mock_subroutine_service() -> MockSubroutinesService {
        MockSubroutinesService::default()
    }
}
