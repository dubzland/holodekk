use std::path::PathBuf;

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};

use nix::unistd::Gid;

use users::get_group_by_name;

use holodekkd::api;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Options {
    /// Path to the unix socket
    #[arg(
        long = "socket",
        value_name = "file",
        default_value = "/var/run/holodekk.sock"
    )]
    socket_path: PathBuf,

    /// Group for the unix socket (default: root)
    #[arg(
        short = 'G',
        long = "group",
        value_name = "group",
        default_value = "docker"
    )]
    socket_group: String,
}

#[actix_web::main]
async fn main() -> api::server::InitResult {
    let options = Options::parse();

    // map the requested group name to id
    let socket_gid = match get_group_by_name(&options.socket_group) {
        Some(group) => Gid::from_raw(group.gid()),
        None => {
            let mut cmd = Options::command();
            cmd.error(
                ErrorKind::InvalidValue,
                format!("group {} not found.", &options.socket_group).to_string(),
            )
            .exit();
        }
    };

    api::server::run(socket_gid, &options.socket_path).await
}
