use std::fs;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::Parser;
use log::debug;

use holodekk::{
    config::{HolodekkApiConfig, HolodekkConfig},
    core::{
        projectors::services::ProjectorsService,
        subroutine_definitions::services::SubroutineDefinitionsService,
        subroutines::services::SubroutinesService,
    },
    repositories::{
        memory::{MemoryDatabase, MemoryRepository},
        RepositoryKind,
    },
    servers::director::{DirectorRequest, DirectorServer},
    utils::{
        servers::start_http_server,
        signals::{SignalKind, Signals},
        ConnectionInfo,
    },
};

use holodekkd::api::{router, ApiState};
use holodekkd::config::HolodekkdConfig;
use holodekkd::errors::HolodekkError;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Options {
    /// Data root path
    #[arg(long, default_value = "/var/lib/holodekk")]
    data_root: PathBuf,

    /// Exec root path
    #[arg(long, default_value = "/run/holodekk")]
    exec_root: PathBuf,

    /// Holodekk bin directory
    #[arg(long, default_value = "/usr/local/bin/")]
    bin_path: PathBuf,

    /// Path to holodekk daemon pid file
    #[arg(long, value_name = "file", required = true)]
    pidfile: PathBuf,

    /// Port to listen on
    #[arg(long, short)]
    port: Option<u16>,

    /// Listen address (IP)
    #[arg(long)]
    address: Option<Ipv4Addr>,

    /// Listen socket (UDS)
    #[arg(long, conflicts_with_all = ["port", "address"])]
    socket: Option<PathBuf>,

    /// Repository type
    #[arg(long, value_enum, default_value = "memory")]
    repository: RepositoryKind,
}

fn ensure_directory<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    if !path.as_ref().exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> std::result::Result<(), HolodekkError> {
    let options = Options::parse();

    let api_config = ConnectionInfo::from_options(
        options.port.as_ref(),
        options.address.as_ref(),
        options.socket,
    )
    .unwrap();

    let holodekkd_config = Arc::new(HolodekkdConfig::new(
        &options.data_root,
        &options.exec_root,
        &options.bin_path,
        api_config,
        options.repository,
    ));

    env_logger::init();

    debug!(
        "Starting HolodekkServer with config: {:?}",
        holodekkd_config
    );

    // ensure required paths exist
    ensure_directory(holodekkd_config.projectors_root())?;
    ensure_directory(holodekkd_config.subroutines_root())?;

    let repo = match holodekkd_config.repo_kind() {
        RepositoryKind::Memory => {
            let db = MemoryDatabase::new();
            Arc::new(MemoryRepository::new(Arc::new(db)))
        }
    };

    // start the director server
    let (director_handle, director_sender) =
        DirectorServer::start(holodekkd_config.clone(), repo.clone());

    // start the api server
    let projectors_service = Arc::new(ProjectorsService::new(
        repo.clone(),
        director_sender.clone(),
    ));
    let definitions_service =
        Arc::new(SubroutineDefinitionsService::init(holodekkd_config.clone()).unwrap());
    let subroutines_service = Arc::new(SubroutinesService::new(
        repo.clone(),
        director_sender.clone(),
        projectors_service.clone(),
        definitions_service.clone(),
    ));
    let state = ApiState::new(
        projectors_service,
        definitions_service,
        subroutines_service,
        holodekkd_config.clone(),
    );
    let mut api_server = start_http_server(
        holodekkd_config.holodekk_api_config(),
        router(Arc::new(state)),
    );

    let signal = Signals::new().await;
    match signal {
        SignalKind::Int => {
            debug!("SIGINT received.  Processing shutdown.");
            let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
            api_server.stop().await.unwrap();
            director_sender
                .send(DirectorRequest::Shutdown { resp: resp_tx })
                .await
                .unwrap();
            drop(director_sender);
            resp_rx.await.unwrap().unwrap();

            director_handle.await.unwrap();
            debug!("received shutdown response.");
        }
        SignalKind::Quit | SignalKind::Term => {
            debug!("Unexpected {} received.  Terminating immediately", signal);
        }
    }

    Ok(())
}
