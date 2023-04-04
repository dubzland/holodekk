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
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Repository {
    async fn get_subroutine_by_name(&'life0 self, name: &str) -> Result<Subroutine>;
    async fn create_subroutine(&'life0 self, subroutine: &Subroutine) -> Result<Subroutine>;
}
