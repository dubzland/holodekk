pub mod containers;
pub mod projectors;
pub mod repositories;
pub mod subroutine_definitions;
pub mod subroutines;

pub mod services {
    use async_trait::async_trait;
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
        #[error("General Network Error")]
        Network,
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
                Error::Network => Self::internal("General Network Error"),
            }
        }
    }

    impl From<tonic::transport::Error> for Error {
        fn from(_err: tonic::transport::Error) -> Error {
            Error::Network
        }
    }

    pub type Result<T> = std::result::Result<T, Error>;

    #[async_trait]
    pub trait ServiceStop: Send + Sync {
        async fn stop(&self) -> crate::core::services::Result<()>;
    }
}
