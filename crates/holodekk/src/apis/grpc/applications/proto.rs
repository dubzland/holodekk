mod pb {
    pub(crate) mod common {
        tonic::include_proto!("common");
    }

    pub(crate) mod applications {
        tonic::include_proto!("applications");
    }
}

pub mod entities {
    pub use super::pb::applications::RpcListReply;
    pub use super::pb::common::RpcEmpty;
}

pub use pb::applications::rpc_applications_client::RpcApplicationsClient;
pub use pb::applications::rpc_applications_server::{RpcApplications, RpcApplicationsServer};
