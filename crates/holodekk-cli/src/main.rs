use std::path::PathBuf;

use clap::{Parser, Subcommand};

use dialoguer::Confirm;

// use colored::*;

// use holodekk_engine::{docker, ImageKind, Store};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    /// Port used by the Holodekk
    #[arg(short, long, default_value = "6141")]
    port: u16,

    /// Root directory
    #[arg(long)]
    root: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Get the current holodekk status
    #[command(name = "status")]
    Status {},

    /// Build the subroutine in the current directory.
    Build {
        /// Directory where the subroutine is located.
        #[arg(short, long, default_value = "holodekk")]
        directory: String,

        /// Name of the subroutine to run
        #[arg(default_value = "default")]
        name: String,
    },

    Run {
        /// Directory where the subroutine is located.
        #[arg(short, long, default_value = "holodekk")]
        directory: String,

        /// Name of the subroutine to run
        #[arg(default_value = "default")]
        name: String,
    },
}

#[tokio::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    let options = Options::parse();

    let root_directory = ensure_root_directory(&options.root)?;
    if root_directory.is_none() {
        return Ok(());
    }

    match &options.command {
        Commands::Status {} => {
            let mut socket_path = root_directory.as_ref().unwrap().clone();
            socket_path.push("holodekk.sock");
            if socket_path.exists() {
                // do some stuff
            } else {
                eprintln!("Holodekk is not currently running.");
            }
            // let engine = docker::Docker::new();
            // let images = engine.images(ImageKind::Subroutine).await.unwrap();
            // if !images.is_empty() {
            //     println!("{}\n", "Available Subroutines".green());
            //     println!("{:25} {:15}", "Name".bold(), "Tag".bold());
            //     println!("{:-<25} {:-<15}", "", "");
            //     for image in images {
            //         for tag in image.tags().iter() {
            //             println!("{:25} {:15}", image.name(), tag.name());
            //         }
            //     }
            //     println!();
            // }
        }
        Commands::Build { .. } => {
            // runtime::detect(directory, name).unwrap().build().await
        }
        Commands::Run { .. } => {
            // match runtime::detect(directory, name) {
            //     Ok(runtime) => {
            //         runtime.run().await?;
            //     }
            //     Err(err) => match err {
            //         CliRuntimeError::ArgumentError(reason) => {
            //             eprintln!("{}", reason);
            //         }
            //         _ => {
            //             eprintln!("Unknown error.");
            //         }
            //     },
            // }
        }
    };

    Ok(())
}

fn ensure_root_directory(path: &Option<PathBuf>) -> std::io::Result<Option<PathBuf>> {
    let root_directory = if path.is_some() {
        path.as_ref().unwrap().to_owned()
    } else {
        let mut path = dirs::home_dir().unwrap();
        path.push(".holodekk");
        path
    };

    if !root_directory.exists() {
        if Confirm::new()
            .with_prompt(format!(
                "Root directory [{}] does not exist.  Create it?",
                root_directory.to_str().unwrap()
            ))
            .interact()?
        {
            std::fs::create_dir(&root_directory)?;
            Ok(Some(root_directory))
        } else {
            Ok(None)
        }
    } else {
        Ok(Some(root_directory))
    }
}
