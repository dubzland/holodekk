pub mod memory;

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use super::entities::{Projector, Subroutine};

#[derive(thiserror::Error, Clone, Copy, Debug, PartialEq)]
pub enum Error {
    #[error("General Error")]
    General,
    #[error("Entity not found")]
    NotFound,
    #[error("Record already exists")]
    AlreadyExists,
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ProjectorRepository: Send + Sync + 'static {
    async fn projector_create(&self, projector: Projector) -> Result<Projector>;
    async fn projector_get(&self, id: &str) -> Result<Projector>;
    async fn projector_delete(&self, id: &str) -> Result<()>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait SubroutineRepository: Send + Sync + 'static {
    async fn subroutine_create(&self, subroutine: Subroutine) -> Result<Subroutine>;
    async fn subroutine_get(&self, id: &str, include_instances: bool) -> Result<Subroutine>;
    async fn subroutine_get_by_name(
        &self,
        name: &str,
        include_instances: bool,
    ) -> Result<Subroutine>;
}

#[cfg(test)]
pub(crate) mod fixtures {
    use rstest::*;

    use super::MockProjectorRepository;
    use super::MockSubroutineRepository;

    #[fixture]
    pub(crate) fn projector_repository() -> MockProjectorRepository {
        MockProjectorRepository::default()
    }

    #[fixture]
    pub(crate) fn subroutine_repository() -> MockSubroutineRepository {
        MockSubroutineRepository::default()
    }
}
