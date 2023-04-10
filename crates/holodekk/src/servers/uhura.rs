use std::path::PathBuf;
use std::sync::Arc;

use tokio::{
    sync::oneshot::{channel, Sender},
    task::JoinHandle,
};

use crate::{
    apis::grpc::{subroutines::subroutines_api_server, uhura::uhura_api_server},
    repositories::Repository,
    services::{SubroutinesService, UhuraService},
    utils::ConnectionInfo,
    HolodekkConfig,
};

use super::run_server;

pub struct UhuraServer<T>
where
    T: Repository,
{
    config: Arc<HolodekkConfig>,
    namespace: String,
    repository: Arc<T>,
    root: PathBuf,
    server_shutdown: Option<Sender<()>>,
    server_handle: Option<JoinHandle<std::result::Result<(), tonic::transport::Error>>>,
}

impl<T> UhuraServer<T>
where
    T: Repository,
{
    pub fn new<S, P>(config: Arc<HolodekkConfig>, namespace: S, repository: Arc<T>, root: P) -> Self
    where
        S: AsRef<str> + Into<String>,
        P: Into<PathBuf>,
    {
        Self {
            config,
            namespace: namespace.into(),
            repository,
            root: root.into(),
            server_shutdown: None,
            server_handle: None,
        }
    }

    pub fn start(&mut self, listener_config: ConnectionInfo) {
        let (server_shutdown, shutdown_rx) = channel();

        let uhura_service = Arc::new(UhuraService::new());
        let subroutines_service = Arc::new(SubroutinesService::new(
            self.config.clone(),
            self.repository.clone(),
            &self.namespace,
            &self.root,
        ));
        let uhura_server = tonic::transport::Server::builder()
            .add_service(uhura_api_server(uhura_service))
            .add_service(subroutines_api_server(subroutines_service));

        let server_handle =
            tokio::spawn(async { run_server(listener_config, uhura_server, shutdown_rx).await });

        self.server_shutdown = Some(server_shutdown);
        self.server_handle = Some(server_handle);
    }

    pub async fn stop(self) {
        self.server_shutdown.unwrap().send(()).unwrap();
        self.server_handle.unwrap().await.unwrap().unwrap();
    }
}
