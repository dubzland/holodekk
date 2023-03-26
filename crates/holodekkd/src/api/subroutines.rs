use actix_web::{error, get, post, web, web::Bytes, Error, HttpResponse, Responder, Result};

use super::server::ApiServices;

use holodekk::engine::ImageStore;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(images).service(build);
}

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
    Ok(HttpResponse::Ok().body(""))
}
