use clap::{Parser, Subcommand};

use crate::subroutine::Subroutine;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Build a container from a Dockerfile
    Build {
        #[arg(long)]
        projector_port: u16,
    },
    Start {
        #[arg(long)]
        projector_port: u16,
    },
    Manifest {},
}

pub trait Platform {
    fn build(&self, subroutine: &str);
    fn run(&self, subroutine: &str, projector_port: u16);
    fn manifest(&self, subroutine: &str) -> String;
}

/// A single holodekk instance wrapped around a subroutine.
pub struct Holodekk<T>
where
    T: Platform,
{
    subroutine: Subroutine,
    platform: T,
}

impl<T> Holodekk<T>
where
    T: Platform,
{
    pub fn new(subroutine: Subroutine, platform: T) -> Self {
        Self {
            subroutine,
            platform,
        }
    }

    pub fn run(&self, args: &Vec<String>) {
        let options = Options::parse_from(args.iter());

        match &options.command {
            Commands::Build { .. } => {
                self.platform.build(&self.subroutine.name);
            }
            Commands::Manifest {} => {
                let json: String = self.platform.manifest(&self.subroutine.name);
                println!("{}", json);
            }
            Commands::Start { projector_port } => {
                self.platform.run(&self.subroutine.name, *projector_port);
            }
        }
    }
}
