use std::net::SocketAddr;
use std::sync::Arc;

use axum::{response::IntoResponse, routing::get, Extension, Json, Router};

use serde::{Deserialize, Serialize};

use crate::Holodekk;

pub struct ApiServices {
    holodekk: Arc<Holodekk>,
}

impl ApiServices {
    pub fn holodekk(&self) -> Arc<Holodekk> {
        Arc::clone(&self.holodekk)
    }
}

impl ApiServices {}
pub async fn run(holodekk: Arc<Holodekk>, port: u16) {
    // Create the global services
    let services = Arc::new(ApiServices { holodekk });

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let app = Router::new()
        .route("/health", get(health))
        .layer(Extension(services));
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Deserialize, Serialize)]
struct HealthResponse {
    status: String,
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "OK".to_string(),
    })
}
