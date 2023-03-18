use clap::{Parser, Subcommand};

use holodekk_cli::build::{self, Builder};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Build a container from a Dockerfile
    Build {}
}

#[tokio::main]
async fn main() -> build::Result<()> {
    let options = Options::parse();

    match &options.command {
        Commands::Build{} => {
            let mut builder = Builder::new();
            if let Err(e) = builder.build().await {
                println!("{}", e);
            }
        },
    }
    Ok(())
}
