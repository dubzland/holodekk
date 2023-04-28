use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};

use holodekk::core::{repositories::ScenesRepository, scenes_find};

use crate::api::ApiState;

pub async fn handler<T>(
    State(state): State<Arc<ApiState<T>>>,
) -> Result<impl IntoResponse, crate::api::ApiError>
where
    T: ScenesRepository,
{
    let scenes = scenes_find::execute(state.repo(), scenes_find::Request {}).await?;
    Ok(Json(scenes))
}

// #[cfg(test)]
// mod tests {
//     use axum::{
//         body::Body,
//         http::{Request, StatusCode},
//         routing::get,
//         Router,
//     };
//     use rstest::*;
//     use tower::ServiceExt;

//     use crate::core::entities::{fixtures::mock_scene, SceneEntity};
//     use crate::scenes::{fixtures::mock_find_scenes, MockFindScenes};

//     use super::*;

//     fn mock_app(find: MockFindScenes) -> Router {
//         Router::new()
//             .route("/", get(handler))
//             .with_state(Arc::new(ApiState {
//                 services: Arc::new(find),
//             }))
//     }

//     fn make_request(
//         find: MockFindScenes,
//     ) -> tower::util::Oneshot<axum::Router, http::Request<hyper::Body>> {
//         mock_app(find).oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn gets_scenes(mut mock_find_scenes: MockFindScenes) {
//         let find_scenes_result = Ok(vec![]);
//         mock_find_scenes
//             .expect_find_scenes()
//             .return_once(move || find_scenes_result);

//         make_request(mock_find_scenes).await.unwrap();
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn responds_with_ok(mut mock_find_scenes: MockFindScenes) {
//         let find_scenes_result = Ok(vec![]);
//         mock_find_scenes
//             .expect_find_scenes()
//             .return_once(move || find_scenes_result);

//         let response = make_request(mock_find_scenes).await.unwrap();

//         assert_eq!(response.status(), StatusCode::OK);
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn returns_scenes(
//         mut mock_find_scenes: MockFindScenes,
//         mock_scene: SceneEntity,
//     ) {
//         let find_scenes_result = Ok(vec![mock_scene.clone()]);
//         mock_find_scenes
//             .expect_find_scenes()
//             .return_once(move || find_scenes_result);

//         let response = make_request(mock_find_scenes).await.unwrap();

//         let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
//         let p: Vec<SceneEntity> = serde_json::from_slice(&body).unwrap();
//         assert_eq!(p.first().unwrap().id(), mock_scene.id());
//     }
// }
