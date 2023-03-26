use clap::{Parser, Subcommand};

use colored::*;

use holodekk::engine::{docker, ImageKind, Store};

use holodekk_cli::{runtime, CliRuntimeError};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List the available subroutines.
    #[command(name = "ls")]
    List {},

    /// Build the subroutine in the current directory.
    Build {
        /// Directory where the subroutine is located.
        #[arg(short, long, default_value = ".holodekk")]
        directory: String,

        /// Name of the subroutine to run
        #[arg(default_value = "default")]
        name: String,
    },

    Run {
        /// Directory where the subroutine is located.
        #[arg(short, long, default_value = ".holodekk")]
        directory: String,

        /// Name of the subroutine to run
        #[arg(default_value = "default")]
        name: String,
    },
}

#[tokio::main]
async fn main() {
    let options = Options::parse();

    match &options.command {
        Commands::List {} => {
            let engine = docker::Docker::new();
            let images = engine.images(ImageKind::Subroutine).await.unwrap();
            if !images.is_empty() {
                println!("{}\n", "Available Subroutines".green());
                println!("{:25} {:15}", "Name".bold(), "Tag".bold());
                println!("{:-<25} {:-<15}", "", "");
                for image in images {
                    for tag in image.tags().iter() {
                        println!("{:25} {:15}", image.name(), tag.name());
                    }
                }
                println!();
            }
        }
        Commands::Build { directory, name } => {
            runtime::detect(directory, name).unwrap().build().await;
        }
        Commands::Run { directory, name } => match runtime::detect(directory, name) {
            Ok(runtime) => runtime.run().await,
            Err(err) => match err {
                CliRuntimeError::ArgumentError(reason) => {
                    eprintln!("{}", reason);
                }
                _ => {
                    eprintln!("Unknown error.");
                }
            },
        },
    }
}
