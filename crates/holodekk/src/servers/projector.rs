use std::sync::Arc;

use tokio::{
    sync::oneshot::{channel, Sender},
    task::JoinHandle,
};

use crate::{
    apis::grpc::applications::applications_api_server, repositories::ProjectorRepository,
    utils::ConnectionInfo,
};

use super::run_server;

pub struct ProjectorServer<T>
where
    T: ProjectorRepository,
{
    _fleet: String,
    _namespace: String,
    _repository: Arc<T>,
    server_shutdown: Option<Sender<()>>,
    server_handle: Option<JoinHandle<std::result::Result<(), tonic::transport::Error>>>,
}

impl<T> ProjectorServer<T>
where
    T: ProjectorRepository,
{
    pub fn new<S>(fleet: S, namespace: S, repository: Arc<T>) -> Self
    where
        S: AsRef<str> + Into<String>,
    {
        Self {
            _fleet: fleet.into(),
            _namespace: namespace.into(),
            _repository: repository,
            server_shutdown: None,
            server_handle: None,
        }
    }

    pub fn start(&mut self, listener_config: ConnectionInfo) {
        let (server_shutdown, shutdown_rx) = channel();
        let projector_server =
            tonic::transport::Server::builder().add_service(applications_api_server());

        let server_handle = tokio::spawn(async {
            run_server(listener_config, projector_server, shutdown_rx).await
        });

        self.server_shutdown = Some(server_shutdown);
        self.server_handle = Some(server_handle);
    }

    pub async fn stop(self) {
        self.server_shutdown.unwrap().send(()).unwrap();
        self.server_handle.unwrap().await.unwrap().unwrap();
    }
}
