use std::sync::Arc;

use axum::extract::{Path, State};

use crate::core::scene::entity::{
    api::State as SceneState,
    service::{get::Input as GetSceneInput, Get as GetScene},
};
use crate::core::subroutine::entity::{
    api::State as SubroutineState,
    service::{delete::Input, Delete},
};
use crate::entity;
use crate::utils::server::http::DeleteResponse;

/// Delete the given subroutine entity from the server
///
/// # Errors
///
/// - Scene id is invalid (or does not exist)
/// - Subroutine id is invalid (or does not exist)
/// - repository error occurred
pub async fn delete_subroutine<A, E, U>(
    State(state): State<Arc<A>>,
    Path((scene, subroutine)): Path<(String, String)>,
) -> Result<DeleteResponse, entity::service::Error>
where
    A: SubroutineState<U> + SceneState<E>,
    E: GetScene,
    U: Delete,
{
    state
        .scene_entity_service()
        .get(&GetSceneInput::new(&scene))
        .await?;

    state
        .subroutine_entity_service()
        .delete(&Input::new(&subroutine))
        .await?;
    Ok(DeleteResponse)
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::delete,
        Router,
    };
    use mockall::mock;
    use rstest::*;
    use tower::ServiceExt;

    use crate::core::scene::entity::{
        mock_entity as mock_scene_entity,
        service::{mock_get as mock_get_scene, MockGet as MockGetScene},
        Entity as SceneEntity, Id,
    };
    use crate::core::subroutine::{
        entity::{
            mock_entity,
            service::{mock_delete, MockDelete},
        },
        Entity,
    };

    use super::*;

    mock! {
        pub CombinedState<E, U>
            where E: Send + Sync + 'static, U: Send + Sync + 'static {}

        impl<E, U> SubroutineState<U> for CombinedState<E, U>
        where
            E: Send + Sync + 'static,
            U: Send + Sync + 'static,
        {
            fn subroutine_entity_service(&self) -> Arc<U>;
        }
        impl<E, U> SceneState<E> for CombinedState<E, U>
        where
            E: Send + Sync + 'static,
            U: Send + Sync + 'static,
        {
            fn scene_entity_service(&self) -> Arc<E>;
        }
    }

    fn mock_app(mock_get_scene: MockGetScene, mock_delete: MockDelete) -> Router {
        let mut state = MockCombinedState::default();

        state
            .expect_scene_entity_service()
            .return_once(move || Arc::new(mock_get_scene));
        state
            .expect_subroutine_entity_service()
            .return_once(move || Arc::new(mock_delete));
        Router::new()
            .route("/:scene/subroutines/:subroutine", delete(delete_subroutine))
            .with_state(Arc::new(state))
    }

    fn make_request(
        mock_get_scene: MockGetScene,
        mock_delete: MockDelete,
        mock_scene_entity: SceneEntity,
        id: &str,
    ) -> tower::util::Oneshot<axum::Router, http::Request<hyper::Body>> {
        mock_app(mock_get_scene, mock_delete).oneshot(
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
        mock_delete: MockDelete,
        mock_scene_entity: SceneEntity,
    ) {
        mock_get_scene.expect_get().return_once(|input| {
            let id: Id = input.id.parse().unwrap();
            Err(entity::service::Error::NotFound(id))
        });

        let response = make_request(mock_get_scene, mock_delete, mock_scene_entity, "123")
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_not_found_when_subroutine_does_not_exist(
        mut mock_get_scene: MockGetScene,
        mut mock_delete: MockDelete,
        mock_scene_entity: SceneEntity,
        mock_entity: Entity,
    ) {
        {
            let entity = mock_scene_entity.clone();
            mock_get_scene.expect_get().return_once(move |_| Ok(entity));
        }

        {
            let entity = mock_entity.clone();
            mock_delete.expect_delete().return_once(move |_| {
                Err(entity::service::Error::NotFound(entity.id.parse().unwrap()))
            });
        }

        let response = make_request(
            mock_get_scene,
            mock_delete,
            mock_scene_entity,
            &mock_entity.id,
        )
        .await
        .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_no_content(
        mut mock_get_scene: MockGetScene,
        mut mock_delete: MockDelete,
        mock_scene_entity: SceneEntity,
        mock_entity: Entity,
    ) {
        {
            let entity = mock_scene_entity.clone();
            mock_get_scene.expect_get().return_once(move |_| Ok(entity));
        }

        mock_delete.expect_delete().return_once(move |_| Ok(()));

        let response = make_request(
            mock_get_scene,
            mock_delete,
            mock_scene_entity,
            &mock_entity.id,
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}
