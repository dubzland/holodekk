use std::sync::Arc;

use axum::Router;
use log::info;

use holodekk::{
    config::{HolodekkApiConfig, HolodekkConfig},
    core::projectors::{
        self, api::server::ProjectorApiServices, initialize_projectors,
        repositories::ProjectorsRepository, services::ProjectorsService, worker::ProjectorsWorker,
    },
    core::subroutine_definitions::{
        self, api::server::SubroutineDefinitionsApiServices, services::SubroutineDefinitionsService,
    },
    utils::{
        servers::{start_http_server, HttpServerHandle},
        TaskHandle, Worker,
    },
};

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
        info!("stopping Holodekk API server ...");
        self.api_server.stop().await.unwrap();
        info!("stopping Projector worker service ...");
        self.projectors_worker.stop().await;
        Ok(())
    }
}

pub struct ApiServices<R>
where
    R: ProjectorsRepository,
{
    projectors_service: Arc<ProjectorsService<R>>,
    definitions_service: Arc<SubroutineDefinitionsService>,
}

impl<R> ApiServices<R>
where
    R: ProjectorsRepository,
{
    pub fn new(
        projectors_service: Arc<ProjectorsService<R>>,
        definitions_service: Arc<SubroutineDefinitionsService>,
    ) -> Self {
        Self {
            projectors_service,
            definitions_service,
        }
    }
}

impl<R> ProjectorApiServices<ProjectorsService<R>> for ApiServices<R>
where
    R: ProjectorsRepository,
{
    fn projectors(&self) -> Arc<ProjectorsService<R>> {
        self.projectors_service.clone()
    }
}

impl<R> SubroutineDefinitionsApiServices<SubroutineDefinitionsService> for ApiServices<R>
where
    R: ProjectorsRepository,
{
    fn definitions(&self) -> Arc<SubroutineDefinitionsService> {
        self.definitions_service.clone()
    }
}

pub fn router<R>(api_services: Arc<ApiServices<R>>) -> axum::Router
where
    R: ProjectorsRepository + 'static,
{
    Router::new()
        .nest("/", crate::api::router())
        .nest(
            "/projectors",
            projectors::api::server::router(api_services.clone()),
        )
        .nest(
            "/subroutine_definitions",
            subroutine_definitions::api::server::router(api_services),
        )
}

pub async fn start_holodekk_server<C, R>(config: Arc<C>, repo: Arc<R>) -> HolodekkServerHandle
where
    C: HolodekkConfig + HolodekkApiConfig,
    R: ProjectorsRepository + 'static,
{
    info!("starting Projector worker service ...");
    let projectors_worker = projectors::worker::start_worker(config.clone());

    info!("starting Holodekk API server...");
    initialize_projectors(config.clone(), repo.clone())
        .await
        .unwrap();
    let projectors_service = Arc::new(ProjectorsService::new(
        config.clone(),
        repo,
        projectors_worker.sender().unwrap(),
    ));
    let definitions_service = SubroutineDefinitionsService::init(config.clone())
        .expect("Unable to initialize subroutine definitions");
    let api_config = config.holodekk_api_config().clone();
    let api_services = ApiServices::new(projectors_service, Arc::new(definitions_service));
    let api_server = start_http_server(&api_config, router(Arc::new(api_services)));
    HolodekkServerHandle::new(projectors_worker, api_server)
}
