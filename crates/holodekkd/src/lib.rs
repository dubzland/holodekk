pub mod api;
pub mod config;
pub mod errors;
pub mod server;

pub type HolodekkResult<T> = std::result::Result<T, errors::HolodekkError>;
