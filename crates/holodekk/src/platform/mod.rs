use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Manifest {},
}

pub trait Platform {
    fn manifest(&self);
    fn run(&self, args: &[String]) {
        let options = Options::parse_from(args.iter());

        match &options.command {
            Commands::Manifest {} => {
                self.manifest();
            }
        }
    }
}
