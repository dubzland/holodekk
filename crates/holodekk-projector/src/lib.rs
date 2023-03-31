pub mod client;
pub mod errors;
pub mod projector;
pub mod server;

pub(crate) mod hello_world {
    tonic::include_proto!("helloworld");
}

pub use errors::{Error, Result};

// pub use client::ProjectorClient;
// pub use projector::{Projector, ProjectorHandle};
// pub use server::{GreeterServer, MyGreeter, ProjectorServer, Service};
// pub use wrapper::{ServerHandle, ServerManager};
