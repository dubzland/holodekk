use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::apis::http::ApiState;
use crate::core::services::{
    scene::{GetScene, ScenesGetInput},
    subroutine::{DeleteSubroutine, SubroutinesDeleteInput},
    Error,
};

pub async fn delete<A, E, U>(
    State(state): State<Arc<A>>,
    Path((scene, subroutine)): Path<(String, String)>,
) -> Result<impl IntoResponse, Error>
where
    A: ApiState<E, U>,
    E: GetScene,
    U: DeleteSubroutine,
{
    state
        .scenes_service()
        .get(&ScenesGetInput::new(&scene))
        .await?;

    state
        .subroutines_service()
        .delete(&SubroutinesDeleteInput::new(&subroutine))
        .await?;
    Ok((StatusCode::NO_CONTENT, ""))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, routing::delete as http_delete, Router};
    use rstest::*;
    use tower::ServiceExt;

    use crate::apis::http::MockApiState;
    use crate::core::{
        entities::{
            fixtures::{mock_scene_entity, mock_subroutine_entity},
            EntityId, SceneEntity, SubroutineEntity,
        },
        services::{
            scene::{fixtures::mock_get_scene, MockGetScene},
            subroutine::{fixtures::mock_delete_subroutine, MockDeleteSubroutine},
        },
    };

    use super::*;

    fn mock_app(mock_get: MockGetScene, mock_delete: MockDeleteSubroutine) -> Router {
        let mut state = MockApiState::default();

        state
            .expect_scenes_service()
            .return_once(move || Arc::new(mock_get));
        state
            .expect_subroutines_service()
            .return_once(move || Arc::new(mock_delete));
        Router::new()
            .route("/:scene/subroutines/:subroutine", http_delete(delete))
            .with_state(Arc::new(state))
    }

    fn make_request(
        mock_get: MockGetScene,
        mock_delete: MockDeleteSubroutine,
        mock_scene_entity: SceneEntity,
        id: &str,
    ) -> tower::util::Oneshot<axum::Router, http::Request<hyper::Body>> {
        mock_app(mock_get, mock_delete).oneshot(
            Request::builder()
                .method("DELETE")
                .header("Content-Type", "application/json")
                .uri(format!("/{}/subroutines/{}", mock_scene_entity.id, id))
                .body(Body::empty())
                .unwrap(),
        )
    }

    #[rstest]
    #[tokio::test]
    async fn returns_not_found_when_scene_does_not_exist(
        mut mock_get_scene: MockGetScene,
        mock_delete_subroutine: MockDeleteSubroutine,
        mock_scene_entity: SceneEntity,
    ) {
        mock_get_scene.expect_get().return_once(|input| {
            let id: EntityId = input.id.parse().unwrap();
            Err(Error::NotFound(id))
        });

        let response = make_request(
            mock_get_scene,
            mock_delete_subroutine,
            mock_scene_entity,
            "123",
        )
        .await
        .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_not_found_when_subroutine_does_not_exist(
        mut mock_get_scene: MockGetScene,
        mut mock_delete_subroutine: MockDeleteSubroutine,
        mock_scene_entity: SceneEntity,
        mock_subroutine_entity: SubroutineEntity,
    ) {
        {
            let entity = mock_scene_entity.clone();
            mock_get_scene.expect_get().return_once(move |_| Ok(entity));
        }

        {
            let entity = mock_subroutine_entity.clone();
            mock_delete_subroutine
                .expect_delete()
                .return_once(move |_| Err(Error::NotFound(entity.id.parse().unwrap())));
        }

        let response = make_request(
            mock_get_scene,
            mock_delete_subroutine,
            mock_scene_entity,
            &mock_subroutine_entity.id,
        )
        .await
        .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_no_content(
        mut mock_get_scene: MockGetScene,
        mut mock_delete_subroutine: MockDeleteSubroutine,
        mock_scene_entity: SceneEntity,
        mock_subroutine_entity: SubroutineEntity,
    ) {
        {
            let entity = mock_scene_entity.clone();
            mock_get_scene.expect_get().return_once(move |_| Ok(entity));
        }

        mock_delete_subroutine
            .expect_delete()
            .return_once(move |_| Ok(()));

        let response = make_request(
            mock_get_scene,
            mock_delete_subroutine,
            mock_scene_entity,
            &mock_subroutine_entity.id,
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}
