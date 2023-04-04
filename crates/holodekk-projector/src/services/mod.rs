mod subroutines;
pub use subroutines::SubroutinesService;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("Entity already exists")]
    Duplicate,
    #[error("Repository Error")]
    Repository(#[from] super::repository::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
