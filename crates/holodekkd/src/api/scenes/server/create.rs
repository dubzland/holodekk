use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use holodekk::core::{actions::scene_create, enums::SceneStatus, repositories::ScenesRepository};

use crate::api::scenes::models::NewScene;
use crate::api::{ApiError, ApiState};

pub async fn handler<T>(
    State(state): State<Arc<ApiState<T>>>,
    Json(new_scene): Json<NewScene>,
) -> Result<impl IntoResponse, ApiError>
where
    T: ScenesRepository,
{
    let scene = scene_create::execute(
        state.repo(),
        scene_create::Request {
            name: &new_scene.name.into(),
            status: &SceneStatus::Created,
        },
    )
    .await?;
    // let scene = state.services().create_scene(&new_scene.fleet).await?;
    Ok((StatusCode::CREATED, Json(scene)))
}

// #[cfg(test)]
// mod tests {
//     use axum::{body::Body, http::Request, routing::post, Router};
//     use rstest::*;
//     use tower::ServiceExt;

//     use holodekk::core::entities::{fixtures::mock_scene, SceneEntity};
//     use holodekk::scenes::{fixtures::mock_create_scene, CreateSceneError, MockCreateScene};

//     use super::*;

//     fn mock_app(create: MockCreateScene) -> Router {
//         Router::new()
//             .route("/", post(handler))
//             .with_state(Arc::new(ApiState {
//                 services: Arc::new(create),
//             }))
//     }

//     fn make_request(
//         create: MockCreateScene,
//     ) -> tower::util::Oneshot<axum::Router, http::Request<hyper::Body>> {
//         let body = Body::from(
//             serde_json::to_string(&NewScene {
//                 fleet: "test".to_string(),
//             })
//             .unwrap(),
//         );

//         mock_app(create).oneshot(
//             Request::builder()
//                 .method("POST")
//                 .header("Content-Type", "application/json")
//                 .uri("/")
//                 .body(body)
//                 .unwrap(),
//         )
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn responds_with_conflict_when_scene_exists(mut mock_create_scene: MockCreateScene) {
//         let create_result = Err(CreateSceneError::Conflict("test".into()));
//         mock_create_scene
//             .expect_create_scene()
//             .withf(|fleet| fleet == "test")
//             .return_once(move |_| create_result);

//         let response = make_request(mock_create_scene).await.unwrap();

//         assert_eq!(response.status(), StatusCode::CONFLICT);
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn responds_with_created(
//         mut mock_create_scene: MockCreateScene,
//         mock_scene: SceneEntity,
//     ) {
//         let create_result = Ok(mock_scene.clone());
//         mock_create_scene
//             .expect_create_scene()
//             .withf(|fleet| fleet == "test")
//             .return_once(move |_| create_result);

//         let response = make_request(mock_create_scene).await.unwrap();

//         assert_eq!(response.status(), StatusCode::CREATED);
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn returns_the_new_scene(
//         mut mock_create_scene: MockCreateScene,
//         mock_scene: SceneEntity,
//     ) {
//         let create_result = Ok(mock_scene.clone());
//         mock_create_scene
//             .expect_create_scene()
//             .withf(|fleet| fleet == "test")
//             .return_once(move |_| create_result);

//         let response = make_request(mock_create_scene).await.unwrap();

//         let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
//         let p: SceneEntity = serde_json::from_slice(&body).unwrap();
//         assert_eq!(p.id(), mock_scene.id());
//     }
// }
