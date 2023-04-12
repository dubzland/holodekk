use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};

use holodekk::core::repositories::ProjectorRepository;
use holodekk::core::services::projectors::All;

use super::ApiServices;

pub fn routes<T>() -> Router<Arc<ApiServices<T>>>
where
    T: ProjectorRepository,
{
    Router::new().route("/", get(list))
}

async fn list<T>(State(state): State<Arc<ApiServices<T>>>) -> impl IntoResponse
where
    T: ProjectorRepository,
{
    let projectors = state.projectors().all().await.unwrap();
    Json(projectors)
}
