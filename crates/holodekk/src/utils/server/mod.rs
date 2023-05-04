//! API server management utilities.

use async_trait::async_trait;

use super::ConnectionInfo;

/// A wrapper for an instance of a running server.
///
/// Primarly used to issue a shutdown command.
#[async_trait]
pub trait Handle<E> {
    /// Issue a shutdown command to the running server.
    ///
    /// Waits for the server task to complete.
    async fn stop(self) -> std::result::Result<(), E>;
}

/// An instance of a "server" listening via tcp or unix sockets.
pub trait Server<T, E> {
    /// The type of Handle this server returns.
    type Handle: Handle<E>;

    /// Starts the server (via tokio).
    fn start(config: &ConnectionInfo, router: T) -> Self::Handle;
}

pub mod grpc;
pub use grpc::Grpc;
pub mod http;
pub use self::http::Http;
