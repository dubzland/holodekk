use futures::StreamExt;

use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};

use hyper::{Body, Client, Method, Request, body::HttpBody, Uri as HyperUri};
use hyperlocal::{UnixConnector, Uri};

use serde::Deserialize;

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(build)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
