use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};

use crate::apis::http::ApiState;
use crate::services::{
    scene::{GetScene, GetSceneInput},
    subroutine::{FindSubroutines, FindSubroutinesInput},
    EntityServiceError,
};

pub async fn find<A, E, U>(
    State(state): State<Arc<A>>,
    Path(scene): Path<String>,
) -> Result<impl IntoResponse, EntityServiceError>
where
    A: ApiState<E, U>,
    E: GetScene,
    U: FindSubroutines,
{
    let scene = state
        .scene_entity_service()
        .get(&GetSceneInput::new(&scene))
        .await?;

    let subroutines = state
        .subroutine_entity_service()
        .find(&FindSubroutinesInput::new(Some(&scene.id), None))
        .await?;
    Ok(Json(subroutines))
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
    use crate::entities::{
        fixtures::{mock_scene_entity, mock_subroutine_entity},
        EntityId, SceneEntity, SubroutineEntity,
    };
    use crate::services::{
        scene::{fixtures::mock_get_scene, MockGetScene},
        subroutine::{fixtures::mock_find_subroutines, MockFindSubroutines},
    };

    use super::*;

    fn mock_app(mock_get_scene: MockGetScene, mock_find: MockFindSubroutines) -> Router {
        let mut state = MockApiState::default();

        state
            .expect_scene_entity_service()
            .return_once(move || Arc::new(mock_get_scene));
        state
            .expect_subroutine_entity_service()
            .return_once(move || Arc::new(mock_find));
        Router::new()
            .route("/:scene/subroutines/", get(find))
            .with_state(Arc::new(state))
    }

    fn make_request(
        mock_get: MockGetScene,
        mock_find: MockFindSubroutines,
        scene: SceneEntity,
    ) -> tower::util::Oneshot<axum::Router, http::Request<hyper::Body>> {
        mock_app(mock_get, mock_find).oneshot(
            Request::builder()
                .method("GET")
                .header("Content-Type", "application/json")
                .uri(format!("/{}/subroutines/", scene.id))
                .body(Body::empty())
                .unwrap(),
        )
    }

    #[rstest]
    #[tokio::test]
    async fn returns_not_found_when_scene_does_not_exist(
        mut mock_get_scene: MockGetScene,
        mock_find_subroutines: MockFindSubroutines,
        mock_scene_entity: SceneEntity,
    ) {
        mock_get_scene.expect_get().return_once(move |input| {
            let id: EntityId = input.id.parse().unwrap();
            Err(EntityServiceError::NotFound(id))
        });

        let response = make_request(mock_get_scene, mock_find_subroutines, mock_scene_entity)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_ok(
        mut mock_get_scene: MockGetScene,
        mut mock_find_subroutines: MockFindSubroutines,
        mock_scene_entity: SceneEntity,
    ) {
        {
            let entity = mock_scene_entity.clone();
            mock_get_scene.expect_get().return_once(move |_| Ok(entity));
        }

        mock_find_subroutines
            .expect_find()
            .return_once(move |_| Ok(vec![]));

        let response = make_request(mock_get_scene, mock_find_subroutines, mock_scene_entity)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_subroutines(
        mut mock_get_scene: MockGetScene,
        mut mock_find_subroutines: MockFindSubroutines,
        mock_scene_entity: SceneEntity,
        mock_subroutine_entity: SubroutineEntity,
    ) {
        {
            let entity = mock_scene_entity.clone();
            mock_get_scene.expect_get().return_once(move |_| Ok(entity));
        }

        {
            let entity = mock_subroutine_entity.clone();
            mock_find_subroutines
                .expect_find()
                .return_once(move |_| Ok(vec![entity]));
        }

        let response = make_request(mock_get_scene, mock_find_subroutines, mock_scene_entity)
            .await
            .unwrap();

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let p: Vec<SubroutineEntity> = serde_json::from_slice(&body).unwrap();
        assert_eq!(p.first().unwrap(), &mock_subroutine_entity);
    }
}
