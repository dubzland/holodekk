pub mod entities {
    pub use crate::proto::applications::RpcListReply;
    pub use crate::proto::common::RpcEmpty;
}

pub use crate::proto::applications::rpc_applications_client::RpcApplicationsClient;
pub use crate::proto::applications::rpc_applications_server::{
    RpcApplications, RpcApplicationsServer,
};
