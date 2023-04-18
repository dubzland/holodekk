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
        services::ServiceStop,
        subroutine_definitions::{
            self, api::server::SubroutineDefinitionsApiServices,
            services::SubroutineDefinitionsService,
        },
        subroutines::{
            self, api::server::SubroutinesApiServices, repositories::SubroutinesRepository,
            CreateSubroutine, FindSubroutines,
        },
        ApiCoreState,
    },
    utils::servers::{start_http_server, HttpServerHandle},
};

use super::errors::HolodekkError;

pub struct HolodekkServerHandle<P, S>
where
    P: ServiceStop,
    S: ServiceStop,
{
    projectors_service: Arc<P>,
    subroutines_service: Arc<S>,
    api_server: HttpServerHandle,
}

impl<P, S> HolodekkServerHandle<P, S>
where
    P: ServiceStop,
    S: ServiceStop,
{
    fn new(
        projectors_service: Arc<P>,
        subroutines_service: Arc<S>,
        api_server: HttpServerHandle,
    ) -> Self {
        Self {
            projectors_service,
            subroutines_service,
            api_server,
        }
    }

    pub async fn stop(&mut self) {
        info!("stopping Holodekk API server ...");
        self.api_server.stop().await.unwrap();
        info!("stopping Projector worker service ...");
        self.projectors_service.stop().await;
        info!("stopping Subroutines worker service ...");
        self.subroutines_service.stop().await;
    }
}

pub struct ApiState<P, S, C>
where
    P: CreateProjector + DeleteProjector + FindProjectors + GetProjector,
    S: CreateSubroutine + FindSubroutines,
    C: HolodekkConfig,
{
    projectors_service: Arc<P>,
    subroutines_service: Arc<S>,
    definitions_service: Arc<SubroutineDefinitionsService>,
    config: Arc<C>,
}

impl<P, S, C> ApiState<P, S, C>
where
    P: CreateProjector + DeleteProjector + FindProjectors + GetProjector,
    S: CreateSubroutine + FindSubroutines,
    C: HolodekkConfig,
{
    pub fn new(
        projectors_service: Arc<P>,
        subroutines_service: Arc<S>,
        definitions_service: Arc<SubroutineDefinitionsService>,
        config: Arc<C>,
    ) -> Self {
        Self {
            projectors_service,
            subroutines_service,
            definitions_service,
            config,
        }
    }
}

impl<P, S, C> ApiCoreState<C> for ApiState<P, S, C>
where
    P: CreateProjector + DeleteProjector + FindProjectors + GetProjector,
    S: CreateSubroutine + FindSubroutines,
    C: HolodekkConfig,
{
    fn config(&self) -> Arc<C> {
        self.config.clone()
    }
}

impl<P, S, C> ProjectorApiServices<P> for ApiState<P, S, C>
where
    P: CreateProjector + DeleteProjector + FindProjectors + GetProjector,
    S: CreateSubroutine + FindSubroutines,
    C: HolodekkConfig,
{
    fn projectors(&self) -> Arc<P> {
        self.projectors_service.clone()
    }
}

impl<P, S, C> SubroutineDefinitionsApiServices<SubroutineDefinitionsService> for ApiState<P, S, C>
where
    P: CreateProjector + DeleteProjector + FindProjectors + GetProjector,
    S: CreateSubroutine + FindSubroutines,
    C: HolodekkConfig,
{
    fn definitions(&self) -> Arc<SubroutineDefinitionsService> {
        self.definitions_service.clone()
    }
}

impl<P, S, C> SubroutinesApiServices<S> for ApiState<P, S, C>
where
    P: CreateProjector + DeleteProjector + FindProjectors + GetProjector,
    S: CreateSubroutine + FindSubroutines,
    C: HolodekkConfig,
{
    fn subroutines(&self) -> Arc<S> {
        self.subroutines_service.clone()
    }
}

pub fn router<P, S, C>(api_services: Arc<ApiState<P, S, C>>) -> axum::Router
where
    P: CreateProjector + DeleteProjector + FindProjectors + GetProjector + Send + Sync + 'static,
    S: CreateSubroutine + FindSubroutines + Send + Sync + 'static,
    C: HolodekkConfig,
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
) -> std::result::Result<HolodekkServerHandle<impl ServiceStop, impl ServiceStop>, HolodekkError>
where
    C: HolodekkConfig + HolodekkApiConfig,
    R: ProjectorsRepository + SubroutinesRepository + 'static,
{
    let definitions_service =
        Arc::new(subroutine_definitions::create_service(config.clone()).await?);

    info!("starting Projector service ...");
    let projectors_service =
        Arc::new(projectors::create_service(config.clone(), repo.clone()).await?);

    info!("starting Subroutine service ...");
    let subroutines_service = Arc::new(
        subroutines::create_service(config.clone(), definitions_service.clone(), repo.clone())
            .await?,
    );

    info!("starting Holodekk API server...");
    let api_config = config.holodekk_api_config().clone();

    let api_state = ApiState::new(
        projectors_service.clone(),
        subroutines_service.clone(),
        definitions_service,
        config,
    );

    let api_server = start_http_server(&api_config, router(Arc::new(api_state)));
    Ok(HolodekkServerHandle::new(
        projectors_service,
        subroutines_service,
        api_server,
    ))
}
