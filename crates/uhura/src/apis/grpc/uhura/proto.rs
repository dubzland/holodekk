mod pb {

    pub(crate) mod uhura {
        #![allow(
            clippy::default_trait_access,
            clippy::missing_errors_doc,
            clippy::wildcard_imports
        )]

        tonic::include_proto!("uhura");
    }
}

pub mod entities {
    pub use super::pb::uhura::{RpcUhuraStatus, RpcUhuraStatusRequest};
}

pub use pb::uhura::rpc_uhura_client::RpcUhuraClient;
pub use pb::uhura::rpc_uhura_server::{RpcUhura, RpcUhuraServer};
