pub mod projectors;
pub mod subroutines;

use tonic::Status;

use crate::core::repositories;

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum Error {
    #[error("Entity already exists")]
    Duplicate,
    #[error("Entity not found")]
    NotFound,
    #[error("Subroutine is already running")]
    AlreadyRunning,
    #[error("Repository Error")]
    Repository(#[from] repositories::Error),
}

impl From<Error> for Status {
    fn from(err: Error) -> Status {
        match err {
            Error::NotFound => Self::not_found(err.to_string()),
            Error::Duplicate => Self::already_exists(err.to_string()),
            Error::AlreadyRunning => Self::already_exists(err.to_string()),
            Error::Repository(err) => Self::internal(err.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
