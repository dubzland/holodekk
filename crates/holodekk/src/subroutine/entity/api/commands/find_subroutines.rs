use std::sync::Arc;

use axum::extract::{Path, State};

use crate::apis::http::{ApiState, GetResponse};
use crate::entity;
use crate::scene::entity::service::{get::Input as GetInput, Get as GetScene};

use crate::subroutine::entity::{
    api::models::Subroutine,
    service::{find::Input as FindInput, Find as FindSubroutines},
};

/// Retrieve a list of subroutine entities from the server
///
/// # Errors
///
/// - Scene id is invalid
/// - repository error occurred
pub async fn find_subroutines<A, E, U>(
    State(state): State<Arc<A>>,
    Path(scene): Path<String>,
) -> Result<GetResponse<Vec<Subroutine>>, entity::service::Error>
where
    A: ApiState<E, U>,
    E: GetScene,
    U: FindSubroutines,
{
    let scene = state
        .scene_entity_service()
        .get(&GetInput::new(&scene))
        .await?;

    let subroutines = state
        .subroutine_entity_service()
        .find(&FindInput::new(Some(&scene.id), None))
        .await?;

    Ok(GetResponse(
        subroutines.into_iter().map(Into::into).collect(),
    ))
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
    use crate::entity;
    use crate::scene::{
        entity::{
            mock_entity as mock_scene_entity,
            service::{mock_get as mock_get_scene, MockGet as MockGetScene},
        },
        Entity as SceneEntity,
    };
    use crate::subroutine::{
        self,
        entity::{
            mock_entity,
            service::{mock_find as mock_find_subroutines, MockFind as MockFindSubroutines},
            Entity,
        },
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
            .route("/:scene/subroutines/", get(find_subroutines))
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
            let id: subroutine::entity::Id = input.id.parse().unwrap();
            Err(entity::service::Error::NotFound(id))
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
        mock_entity: Entity,
    ) {
        {
            let entity = mock_scene_entity.clone();
            mock_get_scene.expect_get().return_once(move |_| Ok(entity));
        }

        {
            let entity = mock_entity.clone();
            mock_find_subroutines
                .expect_find()
                .return_once(move |_| Ok(vec![entity]));
        }

        let response = make_request(mock_get_scene, mock_find_subroutines, mock_scene_entity)
            .await
            .unwrap();

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let p: Vec<Entity> = serde_json::from_slice(&body).unwrap();
        assert_eq!(p.first().unwrap(), &mock_entity);
    }
}
