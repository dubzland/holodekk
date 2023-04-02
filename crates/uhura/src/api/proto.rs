mod uhura_proto {
    tonic::include_proto!("uhura");
}

pub mod entities {
    pub use super::uhura_proto::{RpcEmpty, RpcProjectorStatus, RpcSubroutine, RpcSubroutineList};
}

pub mod core {
    pub use super::uhura_proto::rpc_core_client::RpcCoreClient;
    pub use super::uhura_proto::rpc_core_server::{RpcCore, RpcCoreServer};
}

pub mod subroutines {
    pub use super::uhura_proto::rpc_subroutines_client::RpcSubroutinesClient;
    pub use super::uhura_proto::rpc_subroutines_server::{RpcSubroutines, RpcSubroutinesServer};
}
