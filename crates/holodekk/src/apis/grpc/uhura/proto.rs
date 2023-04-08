mod pb {
    pub(crate) mod common {
        tonic::include_proto!("common");
    }

    pub(crate) mod uhura {
        tonic::include_proto!("uhura");
    }
}

pub mod entities {
    pub use super::pb::common::RpcEmpty;
    pub use super::pb::uhura::RpcUhuraStatus;
}

pub use pb::uhura::rpc_uhura_client::RpcUhuraClient;
pub use pb::uhura::rpc_uhura_server::{RpcUhura, RpcUhuraServer};
