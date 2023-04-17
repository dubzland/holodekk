use std::sync::Arc;

use axum::Router;
use log::info;

use holodekk::{
    config::{HolodekkApiConfig, HolodekkConfig},
    core::{
        projectors::{
            self, api::server::ProjectorApiServices, repositories::ProjectorsRepository,
            CreateProjector, DeleteProjector, FindProjectors, GetProjector,
        },
        services::{self, ServiceStop},
        subroutine_definitions::{
            self, api::server::SubroutineDefinitionsApiServices,
            services::SubroutineDefinitionsService,
        },
    },
    utils::servers::{start_http_server, HttpServerHandle},
};

pub struct HolodekkServerHandle<P>
where
    P: ServiceStop,
{
    projectors_service: Arc<P>,
    api_server: HttpServerHandle,
}

impl<P> HolodekkServerHandle<P>
where
    P: ServiceStop,
{
    fn new(projectors_service: Arc<P>, api_server: HttpServerHandle) -> Self {
        Self {
            projectors_service,
            api_server,
        }
    }

    pub async fn stop(&mut self) -> Result<(), holodekk::core::services::Error> {
        info!("stopping Holodekk API server ...");
        self.api_server.stop().await.unwrap();
        info!("stopping Projector worker service ...");
        self.projectors_service.stop().await?;
        Ok(())
    }
}

pub struct ApiServices<P>
where
    P: CreateProjector + DeleteProjector + FindProjectors + GetProjector,
{
    projectors_service: Arc<P>,
    definitions_service: Arc<SubroutineDefinitionsService>,
}

impl<P> ApiServices<P>
where
    P: CreateProjector + DeleteProjector + FindProjectors + GetProjector,
{
    pub fn new(
        projectors_service: Arc<P>,
        definitions_service: Arc<SubroutineDefinitionsService>,
    ) -> Self {
        Self {
            projectors_service,
            definitions_service,
        }
    }
}

impl<P> ProjectorApiServices<P> for ApiServices<P>
where
    P: CreateProjector + DeleteProjector + FindProjectors + GetProjector,
{
    fn projectors(&self) -> Arc<P> {
        self.projectors_service.clone()
    }
}

impl<P> SubroutineDefinitionsApiServices<SubroutineDefinitionsService> for ApiServices<P>
where
    P: CreateProjector + DeleteProjector + FindProjectors + GetProjector,
{
    fn definitions(&self) -> Arc<SubroutineDefinitionsService> {
        self.definitions_service.clone()
    }
}

pub fn router<P>(api_services: Arc<ApiServices<P>>) -> axum::Router
where
    P: CreateProjector + DeleteProjector + FindProjectors + GetProjector + Send + Sync + 'static,
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

pub async fn start_holodekk_server<C, R>(
    config: Arc<C>,
    repo: Arc<R>,
) -> services::Result<HolodekkServerHandle<impl ServiceStop>>
where
    C: HolodekkConfig + HolodekkApiConfig,
    R: ProjectorsRepository + 'static,
{
    info!("starting Projector service ...");
    let projectors_service =
        Arc::new(projectors::create_service(config.clone(), repo.clone()).await?);
    let definitions_service =
        Arc::new(subroutine_definitions::create_service(config.clone()).await?);

    info!("starting Holodekk API server...");
    let api_config = config.holodekk_api_config().clone();

    let api_services = ApiServices::new(projectors_service.clone(), definitions_service);

    let api_server = start_http_server(&api_config, router(Arc::new(api_services)));
    Ok(HolodekkServerHandle::new(projectors_service, api_server))
}
