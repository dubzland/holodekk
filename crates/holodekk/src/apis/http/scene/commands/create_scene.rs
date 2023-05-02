use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::apis::http::scene::models::{NewScene, Scene};
use crate::apis::http::ApiState;
use crate::entities::SceneEntity;
use crate::services::{
    scene::{CreateScene, CreateSceneInput},
    EntityServiceError,
};

struct CreateSceneResponse(SceneEntity);

impl IntoResponse for CreateSceneResponse {
    fn into_response(self) -> Response {
        let scene = Scene::from(self.0);
        (StatusCode::CREATED, Json(scene)).into_response()
    }
}

pub async fn create_scene<A, E, U>(
    State(state): State<Arc<A>>,
    Json(new_scene): Json<NewScene>,
) -> Result<impl IntoResponse, EntityServiceError>
where
    A: ApiState<E, U>,
    E: CreateScene,
    U: Send + Sync + 'static,
{
    let scene = state
        .scene_entity_service()
        .create(&CreateSceneInput {
            name: &new_scene.name,
        })
        .await?;

    Ok(CreateSceneResponse(scene))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, routing::post, Router};
    use rstest::*;
    use tower::ServiceExt;

    use crate::apis::http::MockApiState;
    use crate::entities::{fixtures::mock_scene_entity, SceneEntity};
    use crate::services::{
        scene::{fixtures::mock_create_scene, MockCreateScene},
        subroutine::fixtures::MockSubroutineEntityService,
    };

    use super::*;

    fn mock_app(mock_create: MockCreateScene) -> Router {
        let mut state = MockApiState::<MockCreateScene, MockSubroutineEntityService>::default();
        state
            .expect_scene_entity_service()
            .return_once(move || Arc::new(mock_create));
        Router::new()
            .route("/", post(create_scene))
            .with_state(Arc::new(state))
    }

    fn make_request(
        mock_create: MockCreateScene,
    ) -> tower::util::Oneshot<axum::Router, http::Request<hyper::Body>> {
        let body = Body::from(
            serde_json::to_string(&NewScene {
                name: "test".to_string(),
            })
            .unwrap(),
        );

        mock_app(mock_create).oneshot(
            Request::builder()
                .method("POST")
                .header("Content-Type", "application/json")
                .uri("/")
                .body(body)
                .unwrap(),
        )
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_conflict_when_scene_exists(mut mock_create_scene: MockCreateScene) {
        mock_create_scene
            .expect_create()
            .return_once(move |input| Err(EntityServiceError::NotUnique(input.name.to_string())));

        let response = make_request(mock_create_scene).await.unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_created(
        mut mock_create_scene: MockCreateScene,
        mock_scene_entity: SceneEntity,
    ) {
        {
            let entity = mock_scene_entity.clone();
            mock_create_scene
                .expect_create()
                .return_once(move |_| Ok(entity));
        }

        let response = make_request(mock_create_scene).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_the_new_scene(
        mut mock_create_scene: MockCreateScene,
        mock_scene_entity: SceneEntity,
    ) {
        {
            let entity = mock_scene_entity.clone();
            mock_create_scene
                .expect_create()
                .return_once(move |_| Ok(entity));
        }

        let response = make_request(mock_create_scene).await.unwrap();

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let p: SceneEntity = serde_json::from_slice(&body).unwrap();
        assert_eq!(p.id, mock_scene_entity.id);
    }
}
