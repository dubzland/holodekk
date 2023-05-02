use std::sync::Arc;

use crate::core::entities::{SceneEntity, SceneEntityRepository};

use super::EntityServiceResult;

use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CreateScene: Send + Sync + 'static {
    async fn create<'a>(&self, input: &'a CreateSceneInput<'a>)
        -> EntityServiceResult<SceneEntity>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait DeleteScene: Send + Sync + 'static {
    async fn delete<'a>(&self, input: &'a DeleteSceneInput<'a>) -> EntityServiceResult<()>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait FindScenes: Send + Sync + 'static {
    async fn find<'a>(
        &self,
        input: &'a FindScenesInput<'a>,
    ) -> EntityServiceResult<Vec<SceneEntity>>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetScene: Send + Sync + 'static {
    async fn get<'a>(&self, input: &'a GetSceneInput<'a>) -> EntityServiceResult<SceneEntity>;
}

#[derive(Clone, Debug)]
pub struct CreateSceneInput<'c> {
    pub name: &'c str,
}

impl<'c> CreateSceneInput<'c> {
    pub fn new(name: &'c str) -> Self {
        Self { name }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeleteSceneInput<'d> {
    pub id: &'d str,
}

impl<'d> DeleteSceneInput<'d> {
    pub fn new(id: &'d str) -> Self {
        Self { id }
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct FindScenesInput<'f> {
    pub name: Option<&'f str>,
}

impl<'f> FindScenesInput<'f> {
    pub fn new(name: Option<&'f str>) -> Self {
        Self { name }
    }
}

#[derive(Clone, Debug)]
pub struct GetSceneInput<'g> {
    pub id: &'g str,
}

impl<'g> GetSceneInput<'g> {
    pub fn new(id: &'g str) -> Self {
        Self { id }
    }
}

pub trait SceneEntityServiceMethods:
    // CreateScene + DeleteScene + FindScenes + GetScene + Send + Sync + 'static
    CreateScene + DeleteScene + FindScenes + GetScene
{
}

impl<T> SceneEntityServiceMethods for T where
    // T: CreateScene + DeleteScene + FindScenes + GetScene + Send + Sync + 'static
    T: CreateScene + DeleteScene + FindScenes + GetScene
{
}

#[derive(Debug)]
pub struct SceneEntityService<R>
where
    R: SceneEntityRepository,
{
    repo: Arc<R>,
}

impl<R> SceneEntityService<R>
where
    R: SceneEntityRepository,
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
        pub SceneEntityService {}
        #[async_trait]
        impl CreateScene for SceneEntityService {
            async fn create<'a>(&self, input: &'a CreateSceneInput<'a>) -> EntityServiceResult<SceneEntity>;
        }

        #[async_trait]
        impl DeleteScene for SceneEntityService {
            async fn delete<'a>(&self, input: &'a DeleteSceneInput<'a>) -> EntityServiceResult<()>;
        }

        #[async_trait]
        impl FindScenes for SceneEntityService {
            async fn find<'a>(&self, input: &'a FindScenesInput<'a>) -> EntityServiceResult<Vec<SceneEntity>>;
        }

        #[async_trait]
        impl GetScene for SceneEntityService {
            async fn get<'a>(&self, input: &'a GetSceneInput<'a>) -> EntityServiceResult<SceneEntity>;
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
    pub fn mock_scene_service() -> MockSceneEntityService {
        MockSceneEntityService::default()
    }
}
