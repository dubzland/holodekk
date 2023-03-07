use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use actix_web::{post, web, App, HttpResponse, HttpServer};
use awc::{ClientBuilder, Connector};
use awc_uds::UdsConnector;
use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use nix::unistd::chown;
use users::get_group_by_name;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Options {
    /// Path to the unix socket
    #[arg(long = "socket", value_name = "file", default_value = "/var/run/holodekk.sock")]
    socket_path: PathBuf,

    /// Group for the unix socket (default: root)
    #[arg(short = 'G', long = "group", value_name = "group", default_value = "docker")]
    socket_group: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let options = Options::parse();

    // map the requested group name to id
    let socket_gid = match get_group_by_name(&options.socket_group) {
        Some(group) => { nix::unistd::Gid::from_raw(group.gid()) },
        None => {
            let mut cmd = Options::command();
            cmd.error(
                ErrorKind::InvalidValue,
                format!("group {} not found.", &options.socket_group).to_string()
            )
            .exit();
        }
    };

    // initialize the server and bind the socket
    let server = HttpServer::new(|| {
        App::new().service(build)
    })
    .bind_uds(&options.socket_path)?;

    // update socket ownership and permissions
    chown(&options.socket_path, Some(0.into()), Some(socket_gid))?;
    let mut perms = std::fs::metadata(&options.socket_path)?.permissions();
    perms.set_mode(0o660);
    std::fs::set_permissions(&options.socket_path, perms)?;

    server.run().await
}

#[post("/build")]
async fn build(payload: web::Payload) -> HttpResponse {
    println!("Received build request");
    let socket_path = Path::new("/var/run/docker.sock");
    let connector = Connector::new().connector(UdsConnector::new(socket_path));
    let client = ClientBuilder::new().connector(connector).finish();
    println!("Sending build context to Docker");
    let resp = client.post("http://localhost/build").send_stream(payload).await;
    match resp {
        Ok(r) => {
            HttpResponse::Ok().streaming(r)
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Docker error: {}", e))
        }
    }
}
