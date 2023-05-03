use std::sync::Arc;

use crate::subroutine::entity::Repository;

pub trait Methods: Create + Delete + Find + Get {}
impl<T> Methods for T where T: Create + Delete + Find + Get {}

#[derive(Debug)]
pub struct Service<R>
where
    R: Repository,
{
    repo: Arc<R>,
}

impl<R> Service<R>
where
    R: Repository,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

pub mod create;
pub use create::Create;
pub mod delete;
pub use delete::Delete;
pub mod find;
pub use find::Find;
pub mod get;
pub use get::Get;

#[cfg(test)]
pub mod fixtures {
    use async_trait::async_trait;
    use mockall::mock;
    use rstest::*;

    use crate::entity::service::Result;
    use crate::subroutine::Entity;

    pub use super::create::MockCreate;
    pub use super::delete::MockDelete;
    pub use super::find::MockFind;
    pub use super::get::MockGet;
    use super::*;

    mock! {
        pub Service {}
        #[async_trait]
        impl Create for Service {
            async fn create<'a>(&self, input: &'a super::create::Input<'a>) -> Result<Entity>;
        }

        #[async_trait]
        impl Delete for Service {
            async fn delete<'a>(&self, input: &'a super::delete::Input<'a>) -> Result<()>;
        }

        #[async_trait]
        impl Find for Service {
            async fn find<'a>(&self, input: &'a super::find::Input<'a>) -> Result<Vec<Entity>>;
        }

        #[async_trait]
        impl Get for Service {
            async fn get<'a>(&self, input: &'a super::get::Input<'a>) -> Result<Entity>;
        }
    }

    #[fixture]
    pub fn mock_create() -> MockCreate {
        MockCreate::default()
    }

    #[fixture]
    pub fn mock_delete() -> MockDelete {
        MockDelete::default()
    }

    #[fixture]
    pub fn mock_find() -> MockFind {
        MockFind::default()
    }

    #[fixture]
    pub fn mock_get() -> MockGet {
        MockGet::default()
    }

    #[fixture]
    pub fn mock_service() -> MockService {
        MockService::default()
    }
}

#[cfg(test)]
pub use fixtures::*;
