use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;

use holodekk::{Holodekk, HolodekkConfig};

use holodekkd::api;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Options {
    /// Port to listen on
    #[arg(long, short, default_value = "6080")]
    port: u16,

    /// Root path
    #[arg(long, default_value = "/var/lib/holodekk")]
    root: PathBuf,
}

const TEMPORARY_BIN: &str = "/home/jdubz/code/gitlab/holodekk/holodekk/target/debug";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let options = Options::parse();

    let holodekk_config = HolodekkConfig {
        fleet: "local".to_string(),
        root_path: options.root,
        bin_path: TEMPORARY_BIN.into(),
    };

    let holodekk = Arc::new(Holodekk::new(holodekk_config));
    holodekk.init()?;
    api::server::run(holodekk, options.port.to_owned()).await;
    Ok(())
}
