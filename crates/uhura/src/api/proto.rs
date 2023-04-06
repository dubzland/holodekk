pub mod entities {
    pub use crate::proto::entities::{RpcEmpty, RpcProjectorStatus};
}

pub use crate::proto::RpcCoreClient;
pub use crate::proto::{RpcCore, RpcCoreServer};
