use async_trait::async_trait;
use mockall::mock;
use rstest::*;

use holodekk::core::{
    projectors::{self, entities::ProjectorEntity, GetProjector, ProjectorsGetInput},
    subroutine_definitions::entities::{SubroutineDefinitionEntity, SubroutineKind},
    subroutines::{
        self, entities::SubroutineEntity, CreateSubroutine, DeleteSubroutine, FindSubroutines,
        SubroutinesCreateInput, SubroutinesDeleteInput, SubroutinesFindInput,
    },
};

mock! {
    pub GetProjector {}
    #[async_trait]
    impl GetProjector for GetProjector {
        async fn get<'a>(&self, input: &'a ProjectorsGetInput<'a>) -> projectors::Result<ProjectorEntity>;
    }
}

mock! {
    pub CreateSubroutine {}
    #[async_trait]
    impl CreateSubroutine for CreateSubroutine {
        async fn create<'c>(&self, input: &'c SubroutinesCreateInput<'c>) -> subroutines::Result<SubroutineEntity>;
    }
}
mock! {
    pub DeleteSubroutine {}
    #[async_trait]
    impl DeleteSubroutine for DeleteSubroutine {
        async fn delete<'c>(&self, input: &'c SubroutinesDeleteInput<'c>) -> subroutines::Result<()>;
    }
}
mock! {
    pub FindSubroutines {}
    #[async_trait]
    impl FindSubroutines for FindSubroutines {
        async fn find<'a>(&self, input: &'a SubroutinesFindInput<'a>) -> subroutines::Result<Vec<SubroutineEntity>>;
    }
}

#[fixture]
pub fn projector() -> ProjectorEntity {
    ProjectorEntity::new("test", "/tmp/projector")
}

#[fixture]
pub fn subroutine_definition() -> SubroutineDefinitionEntity {
    SubroutineDefinitionEntity::new("test", "/tmp/definition", SubroutineKind::Unknown)
}

#[fixture]
pub fn subroutine(
    projector: ProjectorEntity,
    subroutine_definition: SubroutineDefinitionEntity,
) -> SubroutineEntity {
    SubroutineEntity::build(&projector, &subroutine_definition)
}

#[fixture]
pub fn mock_get_projector() -> MockGetProjector {
    MockGetProjector::default()
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
