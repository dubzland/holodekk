extern crate libc;

use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use holodekk::libsee;
use holodekk::logger;

mod runtime;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the shim's pid file
    #[arg(long = "shim-pidfile", value_name = "file")]
    pidfile: PathBuf,

    /// Path to the runtime (ex. /usr/bin/runc)
    #[arg(long = "runtime-path", value_name = "file")]
    runtime_path: PathBuf,

    /// name for the container instance
    #[arg(long = "container-id", value_name = "id")]
    container_id: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new container
    Create {
        /// Path to the OCI bundle directory
        #[arg(short, long = "bundle", value_name = "dir")]
        bundle_path: PathBuf,
    },
    /// Execute a command in an existing container
    Exec {
        /// Path to the container's pid file
        #[arg(long = "container-pidfile", value_name = "file")]
        container_pidfile: PathBuf,

        runtime_args: Vec<String>,

    }
}

fn main() {
    let options = Cli::parse();

    logger::init("holodekk-shim", log::LevelFilter::Debug);

    let cmd: Box<dyn runtime::Command> = match &options.command {
        Commands::Create { bundle_path } => {
            Box::new(runtime::CreateCommand::new(bundle_path))
        },
        Commands::Exec { container_pidfile, runtime_args } => {
            Box::new(runtime::ExecCommand::new(container_pidfile, runtime_args))
        },
    };

    let shim = runtime::Shim::new(options.runtime_path)
        .container_id(&options.container_id);

    let res = shim.exec(cmd);
    if let Some(pid) = res {
        if let Err(err) = fs::write(&options.pidfile, format!("{}", pid)) {
            panic!("write() to pidfile {} failed: {}", options.pidfile.display(), err);
        }
        libsee::_exit(0);
    }
}
