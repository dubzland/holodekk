use tonic::{Request, Response, Status};

pub mod admin_proto {
    tonic::include_proto!("admin");
}
use admin_proto::subroutine_manager_server::SubroutineManager;
pub use admin_proto::subroutine_manager_server::SubroutineManagerServer;
use admin_proto::{Empty, Subroutine, SubroutineList};

#[derive(Default)]
pub struct AdminService {}

impl AdminService {
    pub fn build() -> tonic::transport::server::Router {
        tonic::transport::Server::builder()
            .add_service(SubroutineManagerServer::new(AdminService::default()))
    }
}

#[tonic::async_trait]
impl SubroutineManager for AdminService {
    async fn list_subroutines(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<SubroutineList>, Status> {
        let sub = Subroutine {
            name: "acme/widgets".to_string(),
            pid: 123,
        };
        Ok(Response::new(SubroutineList {
            subroutines: vec![sub],
        }))
    }
}

// pub struct AdminServer {
//     server_handle: RefCell<Option<JoinHandle<std::result::Result<(), tonic::transport::Error>>>>,
//     cmd_tx: RefCell<Option<Sender<AdminCommand>>>,
// }
