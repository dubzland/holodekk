use std::fs;
// use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::Parser;
use log::debug;

use holodekk::{
    repositories::{
        memory::{MemoryDatabase, MemoryRepository},
        RepositoryKind,
    },
    utils::{
        servers::start_http_server,
        signals::{SignalKind, Signals},
        ConnectionInfo,
    },
};

use holodekkd::config::HolodekkdConfig;

use holodekkd::api::{router, ApiState};
use holodekkd::{Holodekk, HolodekkError};

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

    // let api_config = ConnectionInfo::from_options(
    //     options.port.as_ref(),
    //     options.address.as_ref(),
    //     options.socket,
    // )
    // .unwrap();
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
    ensure_directory(holodekkd_config.projectors_root())?;
    ensure_directory(holodekkd_config.subroutines_root())?;

    let repo = match holodekkd_config.repo_kind() {
        RepositoryKind::Memory => {
            let db = MemoryDatabase::new();
            Arc::new(MemoryRepository::new(Arc::new(db)))
        }
    };

    let holodekk = Holodekk::start().await?;
    let state = ApiState::new(repo.clone());
    let mut api_server = start_http_server(
        holodekkd_config.holodekk_api_config(),
        router(Arc::new(state)),
    );

    let signal = Signals::new().await;
    match signal {
        SignalKind::Int => {
            debug!("SIGINT received.  Processing shutdown.");
            // let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
            // director_sender
            //     .send(DirectorRequest::Shutdown { resp: resp_tx })
            //     .await
            //     .unwrap();
            // drop(director_sender);
            // resp_rx.await.unwrap().unwrap();

            // director_handle.await.unwrap();
            api_server.stop().await.unwrap();
            holodekk.stop().await;
            debug!("received shutdown response.");
        }
        SignalKind::Quit | SignalKind::Term => {
            debug!("Unexpected {} received.  Terminating immediately", signal);
        }
    }

    Ok(())
}
