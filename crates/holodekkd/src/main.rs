use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::Parser;
use log::debug;

use holodekk::{
    entity::{repository, Repository},
    utils::{
        server::{Handle, Http},
        signals, ConnectionInfo, Server, Signals,
    },
};

use holodekkd::api;

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
    repository: repository::Kind,
}

fn ensure_directory<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    if !path.as_ref().exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> std::result::Result<(), holodekkd::Error> {
    let options = Options::parse();

    let api_config = ConnectionInfo::tcp(&options.port, None);

    let holodekkd_config = holodekkd::Config::new(
        &options.data_root,
        &options.exec_root,
        &options.bin_path,
        api_config,
        options.repository,
    );

    env_logger::init();

    debug!(
        "Starting HolodekkServer with config: {:?}",
        holodekkd_config
    );

    // ensure required paths exist
    ensure_directory(holodekkd_config.paths().scenes_root())?;
    ensure_directory(holodekkd_config.paths().subroutines_root())?;

    match holodekkd_config.repo_kind() {
        repository::Kind::Memory => {
            let db = repository::memory::Database::new();
            let repo = Arc::new(repository::Memory::new(Arc::new(db)));
            repo.init().await.unwrap();
            start(repo, holodekkd_config).await
        }
        repository::Kind::Etcd => {
            let etcd = repository::Etcd::new(&["127.0.0.1:2379"]);
            let repo = Arc::new(etcd);
            repo.init().await.unwrap();
            start(repo, holodekkd_config).await
        }
    }
}

async fn start<R>(
    repo: Arc<R>,
    config: holodekkd::Config,
) -> std::result::Result<(), holodekkd::Error>
where
    R: Repository,
{
    let holodekk = holodekkd::Monitor::start(config.clone(), repo.clone()).await?;
    let state = api::HolodekkdApiState::new(repo.clone());
    let api_server = Http::start(config.holodekk_api_config(), api::router(Arc::new(state)));

    let signal = Signals::new().await;
    match signal {
        signals::Kind::Int => {
            debug!("SIGINT received.  Processing shutdown.");

            debug!("Awaiting api server shutdown ...");
            api_server.stop().await.unwrap();
            debug!("API server shutdown complete.");
            debug!("Awaiting Holodekk shutdown ...");
            holodekk.stop().await;
            debug!("Holodekk shutdown complete.");

            debug!("Shutdown complete.");
        }
        signals::Kind::Quit | signals::Kind::Term => {
            debug!("Unexpected {} received.  Terminating immediately", signal);
        }
    }
    Ok(())
}
