use std::env;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use clap::{Parser, Subcommand};

use colored::*;

use holodekk_core::engine::{docker, Image, ImageTag};
use holodekk_core::subroutine;

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
}

#[tokio::main]
async fn main() -> holodekk_core::Result<()> {
    let options = Options::parse();

    let current_dir = env::current_dir().unwrap();
    let mut holodekk_dir = PathBuf::from(current_dir);
    holodekk_dir.push(".holodekk");

    match &options.command {
        Commands::List {} => {
            let docker = docker::Service::new();
            let subroutines = subroutine::Service::new(&docker);
            let images = subroutines.images().await?;
            if images.len() > 0 {
                println!("{}\n", "Available Subroutines".green());
                println!("{:25} {:15}", "Name".bold(), "Tag".bold());
                println!("{:-<25} {:-<15}", "", "");
                for image in images {
                    for tag in image.tags().iter() {
                        println!("{:25} {:15}", image.name(), tag.name());
                    }
                }
                println!("");
            }
        }
        Commands::Build { directory, name } => {
            let current_dir = env::current_dir().unwrap();
            let mut holodekk_dir = PathBuf::from(current_dir);
            holodekk_dir.push(directory);

            if holodekk_dir.try_exists().unwrap() {
                let mut ruby_path = PathBuf::from(&holodekk_dir);
                ruby_path.push(format!("{}.rb", name));
                if ruby_path.try_exists().unwrap() {
                    ProcessCommand::new(&ruby_path)
                        .current_dir(&holodekk_dir)
                        .arg("build")
                        .status()
                        .unwrap();
                } else {
                    println!("Could not find subroutine {}.", ruby_path.display());
                }
            } else {
                println!(
                    "Holodekk directory [{}] does not exist.",
                    holodekk_dir.display()
                );
            }
        }
    }
    Ok(())
}
