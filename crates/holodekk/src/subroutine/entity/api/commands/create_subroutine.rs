use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};

use crate::apis::http::{ApiState, CreateResponse};
use crate::entity::service::Error;
use crate::subroutine::entity::{
    api::models::{NewSubroutine, Subroutine},
    service::{create::Input, Create},
};

/// Creates a new subroutine entity on the server
///
/// # Errors
///
/// - Scene id is invalid (or does not exist)
/// - Input parameters are invalid
/// - repository error occurred
pub async fn create_subroutine<A, E, U>(
    Path(scene): Path<String>,
    State(state): State<Arc<A>>,
    Json(new_subroutine): Json<NewSubroutine>,
) -> Result<CreateResponse<Subroutine>, Error>
where
    A: ApiState<E, U>,
    E: Send + Sync + 'static,
    U: Create,
{
    let subroutine = state
        .subroutine_entity_service()
        .create(&Input::new(&scene, &new_subroutine.image_id))
        .await?;
    Ok(CreateResponse(subroutine.into()))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::post,
        Router,
    };
    use rstest::*;
    use tower::ServiceExt;

    use crate::apis::http::MockApiState;
    use crate::entity;
    use crate::scene::{
        entity::{
            mock_entity as mock_scene_entity, service::MockService as MockSceneEntityService,
        },
        Entity as SceneEntity,
    };
    use crate::subroutine::{
        entity::{
            mock_entity,
            service::{mock_create, MockCreate},
        },
        image::{mock_image, Image},
        Entity,
    };

    use super::*;

    fn mock_app(mock_create: MockCreate) -> Router {
        let mut state = MockApiState::<MockSceneEntityService, MockCreate>::default();
        state
            .expect_subroutine_entity_service()
            .return_once(move || Arc::new(mock_create));
        Router::new()
            .route("/:scene/subroutines", post(create_subroutine))
            .with_state(Arc::new(state))
    }

    fn make_request(
        mock_create: MockCreate,
        scene: SceneEntity,
        image: Image,
    ) -> tower::util::Oneshot<axum::Router, http::Request<hyper::Body>> {
        let body = Body::from(
            serde_json::to_string(&NewSubroutine {
                image_id: image.id.to_string(),
            })
            .unwrap(),
        );

        mock_app(mock_create).oneshot(
            Request::builder()
                .method("POST")
                .header("Content-Type", "application/json")
                .uri(format!("/{}/subroutines", scene.id))
                .body(body)
                .unwrap(),
        )
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_conflict_when_subroutine_exists(
        mut mock_create: MockCreate,
        mock_scene_entity: SceneEntity,
        mock_image: Image,
    ) {
        mock_create
            .expect_create()
            .return_once(move |_| Err(entity::service::Error::NotUnique("Already exists".into())));

        let response = make_request(mock_create, mock_scene_entity, mock_image)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_created(
        mut mock_create: MockCreate,
        mock_scene_entity: SceneEntity,
        mock_image: Image,
        mock_entity: Entity,
    ) {
        {
            let entity = mock_entity.clone();
            mock_create.expect_create().return_once(move |_| Ok(entity));
        }

        let response = make_request(mock_create, mock_scene_entity, mock_image)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_the_new_subroutine(
        mut mock_create: MockCreate,
        mock_entity: Entity,
        mock_scene_entity: SceneEntity,
        mock_image: Image,
    ) {
        {
            let entity = mock_entity.clone();
            mock_create.expect_create().return_once(move |_| Ok(entity));
        }

        let response = make_request(mock_create, mock_scene_entity, mock_image)
            .await
            .unwrap();

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let p: Entity = serde_json::from_slice(&body).unwrap();
        assert_eq!(p, mock_entity);
    }
}
