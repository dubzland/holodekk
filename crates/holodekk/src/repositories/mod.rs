pub mod memory;

use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use super::entities::Subroutine;

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
pub trait Repository: Send + Sync + 'static {
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

    use super::MockRepository;

    #[fixture]
    pub(crate) fn repository() -> MockRepository {
        MockRepository::default()
    }
}
