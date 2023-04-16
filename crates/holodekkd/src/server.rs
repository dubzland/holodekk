use std::sync::Arc;

use axum::Router;
use log::{info, warn};
use nix::{sys::signal::kill, unistd::Pid};

use holodekk::{
    config::{HolodekkApiConfig, HolodekkConfig},
    core::projectors::{
        self,
        api::server::ProjectorApiServices,
        entities::Projector,
        repositories::{ProjectorsQuery, ProjectorsRepository},
        services::ProjectorsService,
        worker::ProjectorsWorker,
    },
    core::subroutine_definitions::{
        api::server::SubroutineDefinitionsApiServices, services::SubroutineDefinitionsService,
    },
    utils::{
        servers::{start_http_server, HttpServerHandle},
        ConnectionInfo, TaskHandle, Worker,
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
        .nest("/projectors", projectors::api::server::router(api_services))
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
    let api_services = ApiServices {
        projectors_service,
        definitions_service: Arc::new(definitions_service),
    };
    let api_server = start_http_server(&api_config, router(Arc::new(api_services)));
    HolodekkServerHandle::new(projectors_worker, api_server)
}

async fn initialize_projectors<C, R>(
    config: Arc<C>,
    repo: Arc<R>,
) -> holodekk::core::services::Result<()>
where
    C: HolodekkConfig + HolodekkApiConfig,
    R: ProjectorsRepository + 'static,
{
    // get the list of running projectors from repository
    let mut repo_projectors = repo.projectors_find(ProjectorsQuery::default()).await?;

    // get the list of actually running projectors
    let mut running_projectors: Vec<Projector> = std::fs::read_dir(config.paths().projectors())
        .unwrap()
        .filter_map(|e| {
            let entry = e.unwrap();
            let mut uhura_pidfile = entry.path();
            uhura_pidfile.push("uhura.pid");
            if uhura_pidfile.try_exists().unwrap() {
                let pid = std::fs::read_to_string(&uhura_pidfile)
                    .expect("Should have been able to read pid file");
                let pid: i32 = pid
                    .parse()
                    .expect("Unable to convert pidfile contents to pid");
                match kill(Pid::from_raw(pid), None) {
                    Err(_) => {
                        info!(
                            "Found existing pidfile at {}, but no process found. Removing directory",
                            uhura_pidfile.display()
                        );
                        warn!("Removing directory: {}", entry.path().display());
                        std::fs::remove_dir_all(entry.path()).unwrap();
                        None
                    }
                    Ok(_) => {
                        let mut uhura_socket = entry.path();
                        uhura_socket.push("uhura.sock");
                        let mut projector_socket = entry.path();
                        projector_socket.push("projector.sock");
                        let namespace = entry.path();
                        let namespace = namespace.iter().last().unwrap().to_str().unwrap();

                        let p = Projector::new(
                            config.fleet(),
                            namespace,
                            &uhura_pidfile,
                            ConnectionInfo::unix(uhura_socket),
                            ConnectionInfo::unix(projector_socket),
                            nix::unistd::Pid::from_raw(pid),
                        );
                        Some(p)
                    }
                }
            } else {
                None
            }
        })
        .collect();

    // synchronize
    while let Some(running) = running_projectors.pop() {
        if let Some(projector) = repo_projectors
            .iter()
            .position(|p| p.pid() == running.pid())
        {
            info!(
                "Found dead projector: {:?} ... removing from repo",
                projector
            );
            repo_projectors.remove(projector);
        } else {
            info!("Found missing projector: {:?} ... adding to repo", running);
            repo.projectors_create(running).await.unwrap();
        }
    }

    // at this point, anything still in repo_projectors isn't running.  trash it.
    for projector in repo_projectors {
        repo.projectors_delete(projector.id()).await.unwrap();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::prelude::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    use holodekk::core::repositories::{
        memory::{MemoryDatabase, MemoryRepository},
        RepositoryKind,
    };
    use holodekk::utils::ConnectionInfo;
    use tempfile::tempdir;

    use crate::config::HolodekkdConfig;

    use super::*;

    #[tokio::test]
    async fn initialize_finds_existing_projector() -> std::io::Result<()> {
        let temp = tempdir().unwrap();
        let root_path = temp.into_path();
        let config = Arc::new(HolodekkdConfig::new(
            "test",
            root_path,
            PathBuf::from("/tmp/bin"),
            ConnectionInfo::unix("/tmp/sock"),
            RepositoryKind::Memory,
        ));

        // create a fake projector
        let mut pidfile = config.paths().projectors().to_owned();
        pidfile.push("local");
        fs::create_dir_all(&pidfile)?;
        pidfile.push("uhura.pid");
        let mut file = File::create(pidfile)?;
        file.write_all(format!("{}", std::process::id()).as_bytes())?;

        let db = Arc::new(MemoryDatabase::new());
        let repo = Arc::new(MemoryRepository::new(db.clone()));

        initialize_projectors(config, repo.clone()).await.unwrap();

        let records = db.projectors().all().unwrap();

        assert!(!records.is_empty());

        Ok(())
    }
}
