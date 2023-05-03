use std::sync::Arc;

use axum::{extract::State, Json};

use crate::apis::http::{ApiState, CreateResponse};
use crate::entity;
use crate::scene::entity::{
    api::models::{NewScene, Scene},
    service::{create::Input, Create},
};

pub async fn create_scene<A, E, U>(
    State(state): State<Arc<A>>,
    Json(new_scene): Json<NewScene>,
) -> Result<CreateResponse<Scene>, entity::service::Error>
where
    A: ApiState<E, U>,
    E: Create,
    U: Send + Sync + 'static,
{
    let scene = state
        .scene_entity_service()
        .create(&Input {
            name: &new_scene.name,
        })
        .await?;

    Ok(CreateResponse(scene.into()))
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
    use crate::scene::{
        self,
        entity::{
            mock_entity,
            service::{mock_create, MockCreate},
        },
    };
    use crate::subroutine::entity::service::MockService as MockSubroutineService;

    use super::*;

    fn mock_app(mock_create: MockCreate) -> Router {
        let mut state = MockApiState::<MockCreate, MockSubroutineService>::default();
        state
            .expect_scene_entity_service()
            .return_once(move || Arc::new(mock_create));
        Router::new()
            .route("/", post(create_scene))
            .with_state(Arc::new(state))
    }

    fn make_request(
        mock_create: MockCreate,
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
    async fn responds_with_conflict_when_scene_exists(mut mock_create: MockCreate) {
        mock_create.expect_create().return_once(move |input| {
            Err(entity::service::Error::NotUnique(input.name.to_string()))
        });

        let response = make_request(mock_create).await.unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_created(mut mock_create: MockCreate, mock_entity: scene::Entity) {
        {
            let entity = mock_entity.clone();
            mock_create.expect_create().return_once(move |_| Ok(entity));
        }

        let response = make_request(mock_create).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_the_new_scene(mut mock_create: MockCreate, mock_entity: scene::Entity) {
        {
            let entity = mock_entity.clone();
            mock_create.expect_create().return_once(move |_| Ok(entity));
        }

        let response = make_request(mock_create).await.unwrap();

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let p: scene::Entity = serde_json::from_slice(&body).unwrap();
        assert_eq!(p.id, mock_entity.id);
    }
}
