use axum::{response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct HealthResponse {
    status: String,
}

pub fn router() -> Router {
    Router::new().route("/health", get(health))
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "OK".to_string(),
    })
}
