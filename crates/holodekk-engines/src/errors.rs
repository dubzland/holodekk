use std::fmt;

use holodekk::errors::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum EngineError {
    #[error("A Bollard error occured.")]
    BollardError(#[from] bollard::errors::Error),
}

impl fmt::Debug for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        error_chain_fmt(self, f)
    }
}
