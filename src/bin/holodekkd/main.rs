use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use futures::StreamExt;
use hyper::{Body, Client, Method, Request, body::HttpBody, Uri as HyperUri};
use hyperlocal::{UnixConnector, Uri};
use nix::unistd::chown;
use serde::Deserialize;
use users::get_group_by_name;

#[derive(Deserialize)]
struct AuxResponse {
    id: String,
}


#[derive(Deserialize)]
enum BuildResponse {
    #[serde(rename = "stream")]
    Stream(String),
    #[serde(rename = "aux")]
    Aux(AuxResponse),
}

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
async fn build(mut payload: web::Payload) -> impl Responder {
    let connector = UnixConnector;
    let uri: HyperUri = Uri::new("/var/run/docker.sock", "/build").into();
    let client: Client<UnixConnector, Body> = Client::builder().build(connector);

    let (sender, body) = Body::channel();

    let producer = async {
        let mut sender = sender;
        while let Some(item) = payload.next().await {
            let chunk = item.unwrap();
            let res = sender.send_data(chunk).await;
            match res {
                Err(_) => break,
                _ => (),
            }
        }
    };

    let docker_req = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("content-type", "application/x-tar")
        .body(body)
        .expect("request builder");

    let (_, resp) = futures::join!(producer, client.request(docker_req));

    match resp {
        Ok(mut r) => {
            println!("Response: {}", r.status());
            while let Some(maybe_chunk) = r.body_mut().data().await {
                let chunk_raw = maybe_chunk.unwrap();
                println!("raw: {:?}", &chunk_raw);
                let utf8 = String::from_utf8(chunk_raw.to_vec()).unwrap();
                let line: BuildResponse = serde_json::from_str(&utf8).unwrap();
                match line {
                    BuildResponse::Stream(msg) => { println!("{}", msg); },
                    BuildResponse::Aux(aux) => { println!("built image: {}" , aux.id); },
                }
            }
            HttpResponse::Ok().body("Hello world!")
        }
        Err(err) => {
            println!("Error: {}", err);
            HttpResponse::InternalServerError().body("Error building image")
        }
    }
}
