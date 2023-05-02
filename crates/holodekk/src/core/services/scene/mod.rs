use std::sync::Arc;

use crate::core::{entities::SceneEntity, repositories::ScenesRepository};

use super::Result;

use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CreateScene: Send + Sync + 'static {
    async fn create<'a>(&self, input: &'a ScenesCreateInput<'a>) -> Result<SceneEntity>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait DeleteScene: Send + Sync + 'static {
    async fn delete<'a>(&self, input: &'a ScenesDeleteInput<'a>) -> Result<()>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait FindScenes: Send + Sync + 'static {
    async fn find<'a>(&self, input: &'a ScenesFindInput<'a>) -> Result<Vec<SceneEntity>>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetScene: Send + Sync + 'static {
    async fn get<'a>(&self, input: &'a ScenesGetInput<'a>) -> Result<SceneEntity>;
}

#[derive(Clone, Debug)]
pub struct ScenesCreateInput<'c> {
    pub name: &'c str,
}

impl<'c> ScenesCreateInput<'c> {
    pub fn new(name: &'c str) -> Self {
        Self { name }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenesDeleteInput<'d> {
    pub id: &'d str,
}

impl<'d> ScenesDeleteInput<'d> {
    pub fn new(id: &'d str) -> Self {
        Self { id }
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct ScenesFindInput<'f> {
    pub name: Option<&'f str>,
}

impl<'f> ScenesFindInput<'f> {
    pub fn new(name: Option<&'f str>) -> Self {
        Self { name }
    }
}

#[derive(Clone, Debug)]
pub struct ScenesGetInput<'g> {
    pub id: &'g str,
}

impl<'g> ScenesGetInput<'g> {
    pub fn new(id: &'g str) -> Self {
        Self { id }
    }
}

pub trait ScenesServiceMethods:
    // CreateScene + DeleteScene + FindScenes + GetScene + Send + Sync + 'static
    CreateScene + DeleteScene + FindScenes + GetScene
{
}

impl<T> ScenesServiceMethods for T where
    // T: CreateScene + DeleteScene + FindScenes + GetScene + Send + Sync + 'static
    T: CreateScene + DeleteScene + FindScenes + GetScene
{
}

#[derive(Debug)]
pub struct ScenesService<R>
where
    R: ScenesRepository,
{
    repo: Arc<R>,
}

impl<R> ScenesService<R>
where
    R: ScenesRepository,
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
        pub ScenesService {}
        #[async_trait]
        impl CreateScene for ScenesService {
            async fn create<'a>(&self, input: &'a ScenesCreateInput<'a>) -> Result<SceneEntity>;
        }

        #[async_trait]
        impl DeleteScene for ScenesService {
            async fn delete<'a>(&self, input: &'a ScenesDeleteInput<'a>) -> Result<()>;
        }

        #[async_trait]
        impl FindScenes for ScenesService {
            async fn find<'a>(&self, input: &'a ScenesFindInput<'a>) -> Result<Vec<SceneEntity>>;
        }

        #[async_trait]
        impl GetScene for ScenesService {
            async fn get<'a>(&self, input: &'a ScenesGetInput<'a>) -> Result<SceneEntity>;
        }
    }

    #[fixture]
    pub fn mock_create_scene() -> MockCreateScene {
        MockCreateScene::default()
    }

    #[fixture]
    pub fn mock_delete_scene() -> MockDeleteScene {
        MockDeleteScene::default()
    }

    #[fixture]
    pub fn mock_find_scenes() -> MockFindScenes {
        MockFindScenes::default()
    }

    #[fixture]
    pub fn mock_get_scene() -> MockGetScene {
        MockGetScene::default()
    }

    #[fixture]
    pub fn mock_scene_service() -> MockScenesService {
        MockScenesService::default()
    }
}
