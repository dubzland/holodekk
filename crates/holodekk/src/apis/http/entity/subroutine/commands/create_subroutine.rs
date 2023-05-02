use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};

use crate::apis::http::entity::subroutine::models::{NewSubroutine, Subroutine};
use crate::apis::http::{ApiState, CreateResponse};
use crate::services::{
    subroutine::{CreateSubroutine, CreateSubroutineInput},
    EntityServiceError,
};

pub async fn create_subroutine<A, E, U>(
    Path(scene): Path<String>,
    State(state): State<Arc<A>>,
    Json(new_subroutine): Json<NewSubroutine>,
) -> Result<CreateResponse<Subroutine>, EntityServiceError>
where
    A: ApiState<E, U>,
    E: Send + Sync + 'static,
    U: CreateSubroutine,
{
    let subroutine = state
        .subroutine_entity_service()
        .create(&CreateSubroutineInput::new(
            &scene,
            &new_subroutine.subroutine_image_id,
        ))
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
    use crate::entities::{
        fixtures::mock_scene_entity, fixtures::mock_subroutine_entity, SceneEntity,
        SubroutineEntity,
    };
    use crate::images::{fixtures::mock_subroutine_image, SubroutineImage};
    use crate::services::{
        scene::fixtures::MockSceneEntityService,
        subroutine::{fixtures::mock_create_subroutine, MockCreateSubroutine},
    };

    use super::*;

    fn mock_app(mock_create: MockCreateSubroutine) -> Router {
        let mut state = MockApiState::<MockSceneEntityService, MockCreateSubroutine>::default();
        state
            .expect_subroutine_entity_service()
            .return_once(move || Arc::new(mock_create));
        Router::new()
            .route("/:scene/subroutines", post(create_subroutine))
            .with_state(Arc::new(state))
    }

    fn make_request(
        mock_create: MockCreateSubroutine,
        scene: SceneEntity,
        subroutine: SubroutineImage,
    ) -> tower::util::Oneshot<axum::Router, http::Request<hyper::Body>> {
        let body = Body::from(
            serde_json::to_string(&NewSubroutine {
                subroutine_image_id: subroutine.id.to_string(),
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
        mut mock_create_subroutine: MockCreateSubroutine,
        mock_scene_entity: SceneEntity,
        mock_subroutine_image: SubroutineImage,
    ) {
        mock_create_subroutine
            .expect_create()
            .return_once(move |_| Err(EntityServiceError::NotUnique("Already exists".into())));

        let response = make_request(
            mock_create_subroutine,
            mock_scene_entity,
            mock_subroutine_image,
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_created(
        mut mock_create_subroutine: MockCreateSubroutine,
        mock_scene_entity: SceneEntity,
        mock_subroutine_image: SubroutineImage,
        mock_subroutine_entity: SubroutineEntity,
    ) {
        {
            let entity = mock_subroutine_entity.clone();
            mock_create_subroutine
                .expect_create()
                .return_once(move |_| Ok(entity));
        }

        let response = make_request(
            mock_create_subroutine,
            mock_scene_entity,
            mock_subroutine_image,
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_the_new_subroutine(
        mut mock_create_subroutine: MockCreateSubroutine,
        mock_subroutine_entity: SubroutineEntity,
        mock_scene_entity: SceneEntity,
        mock_subroutine_image: SubroutineImage,
    ) {
        {
            let entity = mock_subroutine_entity.clone();
            mock_create_subroutine
                .expect_create()
                .return_once(move |_| Ok(entity));
        }

        let response = make_request(
            mock_create_subroutine,
            mock_scene_entity,
            mock_subroutine_image,
        )
        .await
        .unwrap();

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let p: SubroutineEntity = serde_json::from_slice(&body).unwrap();
        assert_eq!(p, mock_subroutine_entity);
    }
}
