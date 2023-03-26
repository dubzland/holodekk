// use std::env;
// use std::path::PathBuf;

use actix_web::{error, get, post, web, web::Bytes, Error, HttpResponse, Responder, Result};

// use bollard::Docker;
// use bollard::image::BuildImageOptions;

// use futures_util::pin_mut;
// use futures_util::stream::StreamExt;

// use tokio_stream::{Stream, StreamExt};

use super::server::ApiServices;
use holodekk_core::engine::ImageStore;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(images).service(build);
}

// struct DockerBuilder {
//     docker: Docker,
//     options: BuildImageOptions<String>,
//     data: Bytes,
// }

// impl DockerBuilder {
//     fn into_stream(self) -> impl Stream<Item = Result<Bytes, bollard::errors::Error>> {
//         async_stream::stream! {
//             let mut data_stream = self.docker.build_image(self.options, None, Some(self.data.into()));
//             while let Some(msg) = data_stream.next().await {
//                 match msg {
//                     Ok(data) => {
//                         println!("Received a data packet.");
//                         println!("{:?}", data);
//                     },
//                     Err(err) => {
//                         println!("Received an error.");
//                         println!("{:?}", err);
//                     }
//                 }
//                 yield Ok(Bytes::from(""));
//             }
//         }
//     }
// }

// struct RuntimeHandles {
//     _ruby: Option<Cleanup>,
// }

// impl RuntimeHandles {
//     fn for_ruby(handle: Cleanup) -> Self {
//         Self { _ruby: Some(handle) }
//     }
// }

#[get("/images")]
async fn images(services: web::Data<ApiServices>) -> Result<impl Responder> {
    let images = services
        .store()
        .subroutine_images()
        .await
        .map_err(|_e| error::ErrorInternalServerError("Bogus"))?;
    Ok(web::Json(images))
}

#[post("/build")]
async fn build(_body: Bytes) -> Result<HttpResponse, Error> {
    println!("Received build request.");

    // let _runtime: RuntimeHandles;
    // let current_dir = env::current_dir().unwrap();
    // let mut holodekk_dir = PathBuf::from(current_dir);
    // holodekk_dir.push(".holodekk");

    // if holodekk_dir.try_exists().unwrap() {
    //     let mut ruby_path = PathBuf::from(&holodekk_dir);
    //     ruby_path.push("default.rb");
    //     if ruby_path.try_exists().unwrap() {
    //         let cleanup = init_ruby(&holodekk_dir, &ruby_path).unwrap();
    //         _runtime = RuntimeHandles::for_ruby(cleanup);
    //     }
    // }

    // let options = BuildImageOptions::<String> {
    //     dockerfile: "Dockerfile".to_string(),
    //     t: "holodekk-example".to_string(),
    //     q: true,
    //     ..Default::default()
    // };

    // let docker = Docker::connect_with_socket_defaults().unwrap();
    // let builder = DockerBuilder { docker, options, data: body };
    // let stream = builder.into_stream();
    // let client = docker::Client::new();
    // let stream = client.build("holodekk-example", body);
    // Ok(HttpResponse::Ok().streaming(stream))
    Ok(HttpResponse::Ok().body(""))
}

// fn init_ruby(holodekk_dir: &PathBuf, ruby_file: &PathBuf) -> Result<Cleanup, MagnusError> {
//     let cleanup = unsafe { embed::init() };
//     // let module = define_module("Holodekk")?;
//     // let injector = module.define_class("Injector", class::object())?;
//     // injector.define_singleton_method("inject", function!(handle_subroutine, 1))?;

//     let current_dir = env::current_dir().unwrap();

//     env::set_current_dir(holodekk_dir).unwrap();
//     env::set_var("HOLODEKK_TARGET", ruby_file);
//     require(ruby_file.to_str().unwrap()).unwrap();
//     env::set_current_dir(current_dir).unwrap();

//     // Get a reference to the global Holodekk module
//     if let Some(holodekk) = RModule::from_value(eval("Holodekk").unwrap()) {
//         if holodekk.respond_to("subroutines", false).unwrap() {
//             // Get the subroutine hash
//             let subroutines: RHash = holodekk.funcall("subroutines", ()).unwrap();
//             let keys: RArray = subroutines.funcall("keys", ()).unwrap();
//             println!("subroutines: {}", keys);
//             // let res: RString = holodekk.funcall("subroutines", ())?;
//             // println!("hello: {}", res);
//         }
//     }

//     Ok(cleanup)
// }
