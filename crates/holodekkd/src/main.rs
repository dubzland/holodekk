use std::fs;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::Parser;
use log::debug;

use holodekk::{
    config::HolodekkConfig,
    core::repositories::{
        memory::{MemoryDatabase, MemoryRepository},
        RepositoryKind,
    },
    utils::{
        signals::{SignalKind, Signals},
        ConnectionInfo,
    },
};

use holodekkd::config::HolodekkdConfig;
use holodekkd::errors::HolodekkError;
use holodekkd::server::start_holodekk_server;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Options {
    /// Fleet this holodekk is responsible for
    #[arg(long)]
    fleet: String,

    /// Root path
    #[arg(long, default_value = "/var/lib/holodekk")]
    root_path: PathBuf,

    /// Holodekk bin directory
    #[arg(long, default_value = "/usr/local/bin/")]
    bin_path: PathBuf,

    /// Path to the projector's pid file
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

    let holodekkd_config = HolodekkdConfig::new(
        &options.fleet,
        options.root_path,
        options.bin_path,
        api_config,
        options.repository,
    );

    env_logger::init();

    debug!(
        "Starting HolodekkServer with config: {:?}",
        holodekkd_config
    );

    // ensure required paths exist
    ensure_directory(holodekkd_config.paths().root())?;
    ensure_directory(holodekkd_config.paths().projectors())?;
    ensure_directory(holodekkd_config.paths().subroutines())?;

    let repo = match holodekkd_config.repo_kind() {
        RepositoryKind::Memory => {
            let db = MemoryDatabase::new();
            Arc::new(MemoryRepository::new(Arc::new(db)))
        }
    };

    let mut holodekk = start_holodekk_server(Arc::new(holodekkd_config), repo).await?;

    let signal = Signals::new().await;
    match signal {
        SignalKind::Int => {
            debug!("SIGINT received.  Processing shutdown.");
            holodekk.stop().await.unwrap();
        }
        SignalKind::Quit | SignalKind::Term => {
            debug!("Unexpected {} received.  Terminating immediately", signal);
        }
    }

    Ok(())
}
