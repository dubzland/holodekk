use std::sync::Arc;

use log::debug;

use holodekk::core::projectors::{
    self, repositories::ProjectorsRepository, services::ProjectorsService, worker::ProjectorsWorker,
};
use holodekk::utils::{TaskHandle, Worker};
use holodekk::{
    config::{HolodekkApiConfig, HolodekkConfig},
    servers::{start_http_server, HttpServerHandle},
};

use crate::api::server::router;

pub struct HolodekkServerHandle {
    projectors_worker: ProjectorsWorker,
    api_server: HttpServerHandle,
}

impl HolodekkServerHandle {
    fn new(projectors_worker: ProjectorsWorker, api_server: HttpServerHandle) -> Self {
        Self {
            projectors_worker,
            api_server,
        }
    }

    pub async fn stop(mut self) -> Result<(), tonic::transport::Error> {
        self.api_server.stop().await.unwrap();
        self.projectors_worker.stop().await;
        Ok(())
    }
}

pub fn start_holodekk_server<C, R>(config: Arc<C>, repo: Arc<R>) -> HolodekkServerHandle
where
    C: HolodekkConfig + HolodekkApiConfig,
    R: ProjectorsRepository + 'static,
{
    debug!("starting Projector worker service ...");
    let projectors_worker = projectors::worker::start_worker(config.clone());

    debug!("starting Holodekk API server...");
    let projectors_service = Arc::new(ProjectorsService::new(
        config.clone(),
        repo,
        projectors_worker.sender().unwrap(),
    ));
    let api_config = config.holodekk_api_config().clone();
    let api_server = start_http_server(&api_config, router(projectors_service));
    HolodekkServerHandle::new(projectors_worker, api_server)
}
