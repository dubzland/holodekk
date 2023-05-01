use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::apis::http::ApiState;
use crate::core::{
    models::NewScene,
    services::scene::{CreateScene, Error, ScenesCreateInput},
};

pub async fn create<A, S>(
    State(state): State<Arc<A>>,
    Json(new_scene): Json<NewScene>,
) -> Result<impl IntoResponse, Error>
where
    A: ApiState<S>,
    S: CreateScene,
{
    let scene = state
        .scenes_service()
        .create(&ScenesCreateInput {
            name: &new_scene.name,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(scene)))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, routing::post, Router};
    use rstest::*;
    use tower::ServiceExt;

    use crate::apis::http::MockApiState;
    use crate::core::{
        entities::{fixtures::mock_scene_entity, SceneEntity, SceneName},
        services::scene::{fixtures::mock_create_scene, Error, MockCreateScene},
    };

    use super::*;

    fn mock_app(mock_create: MockCreateScene) -> Router {
        let mut state = MockApiState::default();
        state
            .expect_scenes_service()
            .return_once(move || Arc::new(mock_create));
        Router::new()
            .route("/", post(create))
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
            .return_once(move |input| Err(Error::NotUnique(SceneName::from(input.name))));

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
