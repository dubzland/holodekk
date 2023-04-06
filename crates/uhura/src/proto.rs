pub(crate) mod pb {
    pub(crate) mod uhura {
        tonic::include_proto!("uhura");
    }
}

pub mod entities {
    pub use crate::pb::uhura::{RpcEmpty, RpcProjectorStatus};
}

pub use crate::pb::uhura::rpc_core_client::RpcCoreClient;
pub use crate::pb::uhura::rpc_core_server::{RpcCore, RpcCoreServer};
