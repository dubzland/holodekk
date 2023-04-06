mod pb {
    pub(crate) mod common {
        tonic::include_proto!("common");
    }

    pub(crate) mod subroutines {
        tonic::include_proto!("subroutines");
    }
}

pub mod entities {
    pub use super::pb::common::RpcEmpty;
    pub use super::pb::subroutines::{
        rpc_subroutine::RpcSubroutineStatus, RpcSubroutine, RpcSubroutineList,
    };
}

pub use pb::subroutines::rpc_subroutines_client::RpcSubroutinesClient;
pub use pb::subroutines::rpc_subroutines_server::{RpcSubroutines, RpcSubroutinesServer};
