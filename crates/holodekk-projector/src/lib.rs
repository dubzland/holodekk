mod admin;
mod client;
mod errors;
mod server;
mod wrapper;

pub use errors::{Error, Result};

pub use admin::{AdminService, SubroutineManagerServer};
pub use client::ProjectorClient;
pub use server::{GreeterServer, MyGreeter};
pub use wrapper::{ServerHandle, ServerManager};
