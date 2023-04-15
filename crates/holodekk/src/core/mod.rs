pub mod containers;
pub mod projectors;
pub mod repositories;
pub mod subroutines;
pub mod services {

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
        #[error("Failed to spawn projector")]
        SpawnFailed,
        #[error("Failed to shutdown projector")]
        ShutdownFailed,
        #[error("Repository Error")]
        Repository(#[from] repositories::Error),
        #[error("Projector Error")]
        Projector(#[from] crate::core::projectors::worker::SpawnError),
    }

    impl From<Error> for Status {
        fn from(err: Error) -> Status {
            match err {
                Error::NotFound => Self::not_found(err.to_string()),
                Error::Duplicate => Self::already_exists(err.to_string()),
                Error::AlreadyRunning => Self::already_exists(err.to_string()),
                Error::SpawnFailed => Self::internal("Unable to spawn projector".to_string()),
                Error::ShutdownFailed => Self::internal("Unable to shutdown projector".to_string()),
                Error::Repository(err) => Self::internal(err.to_string()),
                Error::Projector(err) => Self::internal(err.to_string()),
            }
        }
    }

    pub type Result<T> = std::result::Result<T, Error>;
}
