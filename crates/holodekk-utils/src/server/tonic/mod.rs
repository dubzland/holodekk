//! [Tonic](https://docs.rs/tonic/latest/tonic/) specific server utilities.
mod builder;
pub use builder::TonicServerBuilder;
mod handle;
pub use handle::TonicServerHandle;
mod manager;
pub use manager::TonicServerManager;
mod server;
pub use server::TonicServer;
mod service;
pub use service::TonicService;

type TonicResult = std::result::Result<(), tonic::transport::Error>;
