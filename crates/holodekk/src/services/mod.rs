mod subroutines;
pub use subroutines::{SubroutineCreateInput, SubroutinesService};
mod uhura;
pub use uhura::UhuraService;

use tonic::Status;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("Entity already exists")]
    Duplicate,
    #[error("Entity not found")]
    NotFound,
    #[error("Subroutine is already running")]
    AlreadyRunning,
    #[error("Repository Error")]
    Repository(#[from] crate::repositories::Error),
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
