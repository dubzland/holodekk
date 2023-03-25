#[derive(Debug)]
pub enum Error {
    BuildFailed,
    RuntimeDetectFailed,
    ImageNotFound(String),
    TagNotFound(String),
    BollardError(bollard::errors::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
