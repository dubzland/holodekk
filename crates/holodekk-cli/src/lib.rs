use thiserror::Error;

pub mod runtime;

pub trait CliRuntime {
    fn build(&self);
    fn manifest(&self);
    fn run(&self);
}

#[derive(Debug, Error)]
pub enum CliRuntimeError {
    #[error("Invalid argument: {0}")]
    ArgumentError(String),
    #[error("Unknown Error")]
    Unknown,
}
