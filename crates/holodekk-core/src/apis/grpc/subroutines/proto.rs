pub mod entities {
    pub use crate::proto::common::RpcEmpty;
    pub use crate::proto::subroutines::{
        rpc_subroutine::RpcSubroutineStatus, RpcSubroutine, RpcSubroutineList,
    };
}

pub use crate::proto::subroutines::rpc_subroutines_client::RpcSubroutinesClient;
pub use crate::proto::subroutines::rpc_subroutines_server::{RpcSubroutines, RpcSubroutinesServer};
