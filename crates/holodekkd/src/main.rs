use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::Parser;
use log::debug;

use holodekk::{
    core::entities::repository::Repository,
    repositories::{
        etcd::EtcdRepository,
        memory::{MemoryDatabase, MemoryRepository},
        RepositoryKind,
    },
    utils::{
        signals::{SignalKind, Signals},
        ConnectionInfo,
    },
};

use holodekkd::config::HolodekkdConfig;

use holodekkd::api::Server;
use holodekkd::holodekk::{Holodekk, HolodekkError};

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

    /// Holodekk API port
    #[arg(short, long, default_value = "7979")]
    port: u16,

    /// Holodekk API port
    #[arg(long, value_enum)]
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

    let api_config = ConnectionInfo::tcp(&options.port, None);

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
    ensure_directory(holodekkd_config.paths().scenes_root())?;
    ensure_directory(holodekkd_config.paths().subroutines_root())?;

    match holodekkd_config.repo_kind() {
        RepositoryKind::Memory => {
            let db = MemoryDatabase::new();
            let repo = Arc::new(MemoryRepository::new(Arc::new(db)));
            repo.init().await.unwrap();
            start(repo, holodekkd_config).await
        }
        RepositoryKind::Etcd => {
            let etcd = EtcdRepository::new(&["127.0.0.1:2379"]);
            let repo = Arc::new(etcd);
            repo.init().await.unwrap();
            start(repo, holodekkd_config).await
        }
    }
}

async fn start<R>(
    repo: Arc<R>,
    config: Arc<HolodekkdConfig>,
) -> std::result::Result<(), HolodekkError>
where
    R: Repository + 'static,
{
    let holodekk = Holodekk::start(config.clone(), repo.clone()).await?;
    let mut api_server = Server::start(config.holodekk_api_config(), repo.clone());

    let signal = Signals::new().await;
    match signal {
        SignalKind::Int => {
            debug!("SIGINT received.  Processing shutdown.");

            debug!("Awaiting api server shutdown ...");
            api_server.stop().await;
            debug!("API server shutdown complete.");
            debug!("Awaiting Holodekk shutdown ...");
            holodekk.stop().await;
            debug!("Holodekk shutdown complete.");

            debug!("Shutdown complete.");
        }
        SignalKind::Quit | SignalKind::Term => {
            debug!("Unexpected {} received.  Terminating immediately", signal);
        }
    }
    Ok(())
}
