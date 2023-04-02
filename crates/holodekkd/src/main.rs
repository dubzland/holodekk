use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;

use holodekk::{api, Holodekk};

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

    let bin = PathBuf::from(TEMPORARY_BIN);
    let holodekk = Arc::new(Holodekk::new(&options.root, &bin));
    holodekk.init()?;
    api::server::run(holodekk, options.port.to_owned()).await;
    Ok(())
}
