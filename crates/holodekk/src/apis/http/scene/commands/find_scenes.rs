use std::sync::Arc;

use axum::extract::State;

use crate::apis::http::scene::models::Scene;
use crate::apis::http::{ApiState, GetResponse};
use crate::services::{
    scene::{FindScenes, FindScenesInput},
    EntityServiceError,
};

pub async fn find_scenes<A, E, U>(
    State(state): State<Arc<A>>,
) -> Result<GetResponse<Vec<Scene>>, EntityServiceError>
where
    A: ApiState<E, U>,
    E: FindScenes,
    U: Send + Sync + 'static,
{
    let scenes = state
        .scene_entity_service()
        .find(&FindScenesInput::default())
        .await?;

    Ok(GetResponse(scenes.into_iter().map(Into::into).collect()))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use rstest::*;
    use tower::ServiceExt;

    use crate::apis::http::MockApiState;
    use crate::entities::{fixtures::mock_scene_entity, SceneEntity};
    use crate::services::{
        scene::{fixtures::mock_find_scenes, MockFindScenes},
        subroutine::fixtures::MockSubroutineEntityService,
    };

    use super::*;

    fn mock_app(mock_find: MockFindScenes) -> Router {
        let mut state = MockApiState::<MockFindScenes, MockSubroutineEntityService>::default();
        state
            .expect_scene_entity_service()
            .return_once(move || Arc::new(mock_find));
        Router::new()
            .route("/", get(find_scenes))
            .with_state(Arc::new(state))
    }

    fn make_request(
        mock_find: MockFindScenes,
    ) -> tower::util::Oneshot<axum::Router, http::Request<hyper::Body>> {
        mock_app(mock_find).oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
    }

    #[rstest]
    #[tokio::test]
    async fn gets_scenes_from_service(mut mock_find_scenes: MockFindScenes) {
        mock_find_scenes
            .expect_find()
            .return_once(move |_| Ok(vec![]));

        make_request(mock_find_scenes).await.unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_ok(mut mock_find_scenes: MockFindScenes) {
        mock_find_scenes
            .expect_find()
            .return_once(move |_| Ok(vec![]));

        let response = make_request(mock_find_scenes).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_scenes(mut mock_find_scenes: MockFindScenes, mock_scene_entity: SceneEntity) {
        {
            let entities = vec![mock_scene_entity.clone()];
            mock_find_scenes
                .expect_find()
                .return_once(move |_| Ok(entities));
        }

        let response = make_request(mock_find_scenes).await.unwrap();

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let p: Vec<SceneEntity> = serde_json::from_slice(&body).unwrap();
        assert_eq!(p.first().unwrap(), &mock_scene_entity);
    }
}
