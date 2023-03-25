use async_trait::async_trait;

use thiserror::Error;

pub mod runtime;

#[async_trait]
pub trait CliRuntime {
    async fn build(&self);
    fn manifest(&self);
    async fn run(&self);
}

#[derive(Debug, Error)]
pub enum CliRuntimeError {
    #[error("Invalid argument: {0}")]
    ArgumentError(String),
    #[error("Unknown Error")]
    Unknown,
}
