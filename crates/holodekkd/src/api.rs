use std::fmt;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use actix_web::{web, App, HttpServer};

use nix::unistd::{chown, Gid};

use holodekk_core::subroutine;

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
            },
            Error::Chown(reason) => {
                write!(f, "failed to set permissions on the listening socket: {}", reason)
            },
            Error::Group(reason) => {
                write!(f, "group {} not found", &reason)
            },
            Error::Perms(reason) => {
                write!(f, "failed to set permissions on listening socket: {}", reason)
            },
            Error::Start(reason) => {
                write!(f, "failed to start API server: {}", reason)
            },
        }
    }
}

pub type InitResult = Result<(), Error>;

pub async fn run(socket_gid: Gid, socket_path: &PathBuf) -> InitResult {
    // initialize the server and bind the socket
    let server = HttpServer::new(|| {
        App::new()
            .service(web::scope("/subroutines").configure(subroutine::api::routes))
    })
    .bind_uds(socket_path).map_err(|err| Error::Bind(err))?;

    // update socket ownership and permissions
    chown(socket_path, Some(0.into()), Some(socket_gid)).map_err(|err| Error::Chown(err))?;
    let metadata = std::fs::metadata(socket_path).unwrap();
    let mut perms =  metadata.permissions();
    perms.set_mode(0o660);
    std::fs::set_permissions(socket_path, perms).map_err(|err| Error::Perms(err))?;

    server.run().await.map_err(|err| Error::Start(err))
}
