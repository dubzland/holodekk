mod projector_proto {
    tonic::include_proto!("projector");
}

pub mod entities {
    pub use super::projector_proto::{RpcEmpty, RpcListReply};
}

pub mod applications {
    pub use super::projector_proto::rpc_applications_client::RpcApplicationsClient;
    pub use super::projector_proto::rpc_applications_server::{
        RpcApplications, RpcApplicationsServer,
    };
}
