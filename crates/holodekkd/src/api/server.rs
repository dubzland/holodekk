use log::{debug, info};
use std::fmt;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use nix::unistd::{chown, Gid};

use actix_web::{middleware::Logger, web, App, HttpServer};

use holodekk_engine::docker;

use super::subroutines;

#[derive(Debug)]
pub enum Error {
    Bind(std::io::Error),
    Chown(nix::Error),
    Group(String),
    Perms(std::io::Error),
    Start(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Bind(reason) => {
                write!(f, "failed to bind to listening socket: {}", reason)
            }
            Error::Chown(reason) => {
                write!(
                    f,
                    "failed to set permissions on the listening socket: {}",
                    reason
                )
            }
            Error::Group(reason) => {
                write!(f, "group {} not found", &reason)
            }
            Error::Perms(reason) => {
                write!(
                    f,
                    "failed to set permissions on listening socket: {}",
                    reason
                )
            }
            Error::Start(reason) => {
                write!(f, "failed to start API server: {}", reason)
            }
        }
    }
}

pub type InitResult = Result<(), Error>;

pub struct ApiServices {
    engine: docker::Docker,
}

impl ApiServices {
    pub fn engine(&self) -> &docker::Docker {
        &self.engine
    }
}

pub async fn run(socket_gid: Gid, socket_path: &PathBuf) -> InitResult {
    let unix_socket = false;

    // Create the global services
    let engine = docker::Docker::new();
    let services = web::Data::new(ApiServices { engine });

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    debug!("this is a debug {}", "message");
    info!("this is an info");

    // initialize the server and bind the socket
    let builder = HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(Logger::default())
            .app_data(services.clone())
            .service(web::scope("/subroutines").configure(subroutines::routes))
    });

    let server = match unix_socket {
        true => {
            let s = builder.bind_uds(socket_path).map_err(Error::Bind)?;

            // update socket ownership and permissions
            chown(socket_path, Some(0.into()), Some(socket_gid)).map_err(Error::Chown)?;
            let metadata = std::fs::metadata(socket_path).unwrap();
            let mut perms = metadata.permissions();
            perms.set_mode(0o660);
            std::fs::set_permissions(socket_path, perms).map_err(Error::Perms)?;

            s
        }
        false => builder.bind(("0.0.0.0", 6080)).map_err(Error::Bind)?,
    };

    server.run().await.map_err(Error::Start)
}
