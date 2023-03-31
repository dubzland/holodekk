mod core;
mod subroutines;

pub mod admin_proto {
    tonic::include_proto!("admin");
}

pub use self::core::CoreService;
pub use admin_proto::core_client::CoreClient;
pub use admin_proto::core_server::{Core, CoreServer};
pub use admin_proto::subroutines_client::SubroutinesClient;
pub use admin_proto::subroutines_server::{Subroutines, SubroutinesServer};
pub use admin_proto::{Empty, ProjectorStatus, Subroutine, SubroutineList};
pub use subroutines::SubroutinesService;

pub fn router() -> tonic::transport::server::Router {
    tonic::transport::Server::builder()
        .add_service(SubroutinesServer::new(SubroutinesService::default()))
        .add_service(CoreServer::new(CoreService::new()))
}
