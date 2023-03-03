extern crate libc;

use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use log::debug;

use holodekk::libsee;
use holodekk::logger;

mod runtime;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    /// Path to the shim's pid file
    #[arg(long = "shim-pidfile", value_name = "file", required = true)]
    pidfile: PathBuf,

    /// Path to the runtime (ex. /usr/bin/runc)
    #[arg(long = "runtime-path", value_name = "file", required = true)]
    runtime_path: PathBuf,

    /// name for the container instance
    #[arg(long = "container-id", value_name = "id", required = true)]
    container_id: String,

    /// Path to the container's pid file
    #[arg(long = "container-pidfile", value_name = "file", required = true)]
    container_pidfile: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new container
    Create {
        /// Path to the OCI bundle directory
        #[arg(short, long = "bundle", value_name = "dir", required = true)]
        bundle_path: PathBuf,
    },
    /// Execute a command in an existing container
    Exec {
        runtime_args: Vec<String>,
    }
}


fn main() {
    let options = Options::parse();

    logger::init("holodekk-shim", log::LevelFilter::Debug);

    let res = libsee::fork().unwrap();
    if let Some(pid) = res {
        debug!("forked worker with pid: {}", pid);
        if let Err(err) = fs::write(&options.pidfile, format!("{}", pid)) {
            panic!("write() to pidfile {} failed: {}", options.pidfile.display(), err);
        }
        libsee::_exit(0);
    }

    // Create the container
    let mut container = runtime::Container::new(
        &options.container_id,
        &options.container_pidfile,
    );

    let cmd: Box<dyn runtime::Command> = match &options.command {
        Commands::Create { bundle_path } => {
            Box::new(runtime::CreateCommand::new(bundle_path))
        },
        Commands::Exec { runtime_args } => {
            Box::new(runtime::ExecCommand::new(runtime_args))
        },
    };

    runtime::exec(&options.runtime_path, &mut container, cmd);
}
