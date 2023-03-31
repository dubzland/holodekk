mod core;
mod subroutines;

use self::core::CoreService;
use crate::proto::admin::core::CoreServer;
use crate::proto::admin::subroutines::SubroutinesServer;
pub use subroutines::SubroutinesService;

pub fn router() -> tonic::transport::server::Router {
    tonic::transport::Server::builder()
        .add_service(SubroutinesServer::new(SubroutinesService::default()))
        .add_service(CoreServer::new(CoreService::new()))
}
