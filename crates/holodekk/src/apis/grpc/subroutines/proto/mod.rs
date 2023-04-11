mod pb {
    pub(crate) mod subroutines {
        tonic::include_proto!("subroutines");
    }
}

pub mod entities {
    pub use super::pb::subroutines::{
        RpcStatusRequest, RpcSubroutine, RpcSubroutineInstance, RpcSubroutineKind,
        RpcSubroutineStatus, RpcSubroutineStatusCode,
    };
}

pub mod enums {}

pub use pb::subroutines::rpc_subroutines_client::RpcSubroutinesClient;
pub use pb::subroutines::rpc_subroutines_server::{RpcSubroutines, RpcSubroutinesServer};

mod instance;
mod kind;
mod status;
mod subroutine;
