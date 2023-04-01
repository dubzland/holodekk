mod projector_proto {
    tonic::include_proto!("projector");
}

pub mod entities {
    pub use super::projector_proto::{Empty, ListReply};
}

pub mod applications {
    pub use super::projector_proto::applications_client::ApplicationsClient;
    pub use super::projector_proto::applications_server::{Applications, ApplicationsServer};
}
