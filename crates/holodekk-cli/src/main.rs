use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};

use holodekk::{Holodekk, Result};
use holodekk_cli::{runtime, CliRuntimeError};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Project {
        /// Directory where the subroutine is located.
        #[arg(short, long, default_value = "holodekk")]
        directory: String,

        /// Name of the subroutine to run
        #[arg(default_value = "default")]
        name: String,
    },
}

const TEMPORARY_BIN: &str = "/home/jdubz/code/gitlab/holodekk/holodekk/target/debug";

#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::parse();

    // Start a Holodekk
    let holodekk = Arc::new(Holodekk::new(
        &PathBuf::from("~/.holodekk"),
        &PathBuf::from(TEMPORARY_BIN),
    ));
    holodekk.init()?;

    match &options.command {
        Commands::Project { directory, name } => match runtime::detect(holodekk, directory, name) {
            Ok(runtime) => {
                runtime.project().await?;
            }
            Err(err) => match err {
                CliRuntimeError::ArgumentError(reason) => {
                    eprintln!("{}", reason);
                }
                _ => {
                    eprintln!("Unknown error.");
                }
            },
        },
    };

    Ok(())
}
