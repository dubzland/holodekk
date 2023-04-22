pub mod api;
pub mod config;
pub mod errors;

pub type HolodekkResult<T> = std::result::Result<T, errors::HolodekkError>;
