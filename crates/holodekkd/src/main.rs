use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use log::debug;

use holodekk::{
    core::repositories::RepositoryKind,
    utils::{
        signals::{SignalKind, Signals},
        ConnectionInfo,
    },
};

use holodekkd::config::HolodekkdConfig;
use holodekkd::server::HolodekkServer;

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

#[tokio::main]
async fn main() -> std::io::Result<()> {
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

    let holodekk = HolodekkServer::start(Arc::new(holodekkd_config));

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

    //     let holodekk = Arc::new(Holodekk::new(holodekk_config));
    //     holodekk.init()?;
    //     api::server::run(holodekk, options.port.to_owned()).await;
    Ok(())
}
