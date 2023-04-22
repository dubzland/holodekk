use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use holodekk::core::projectors::{GetProjector, ProjectorsGetInput};
use holodekk::core::subroutines::{CreateSubroutine, SubroutinesCreateInput};

use super::SubroutinesApiServices;
use crate::api::subroutines::models::NewSubroutine;
use crate::api::ProjectorsApiServices;

pub async fn handler<A, S, P>(
    State(state): State<Arc<A>>,
    Path(projector): Path<String>,
    Json(new_subroutine): Json<NewSubroutine>,
) -> Result<impl IntoResponse, crate::api::ApiError>
where
    A: SubroutinesApiServices<S> + ProjectorsApiServices<P>,
    S: CreateSubroutine,
    P: GetProjector,
{
    let projector = state
        .projectors()
        .get(&ProjectorsGetInput::new(&projector))
        .await?;

    let subroutine = state
        .subroutines()
        .create(&SubroutinesCreateInput::new(
            projector.id(),
            &new_subroutine.subroutine_definition_id,
        ))
        .await?;
    Ok((StatusCode::CREATED, Json(subroutine)))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, routing::post, Router};
    use mockall::mock;
    use rstest::*;
    use tower::ServiceExt;

    use holodekk::core::projectors::{entities::ProjectorEntity, ProjectorsError};
    use holodekk::core::subroutines::entities::SubroutineEntity;
    use holodekk::core::subroutines::SubroutinesError;

    use crate::api::fixtures::{
        mock_create_subroutine, mock_get_projector, projector, subroutine, MockCreateSubroutine,
        MockGetProjector,
    };

    use super::*;

    mock! {
        pub ApiServices<P, S> {}
        impl<P, S> ProjectorsApiServices<P> for ApiServices<P, S> {
            fn projectors(&self) -> Arc<P>;
        }
        impl<P, S> SubroutinesApiServices<S> for ApiServices<P, S> {
            fn subroutines(&self) -> Arc<S>;
        }
    }

    #[fixture]
    fn mock_services() -> MockApiServices<MockGetProjector, MockCreateSubroutine> {
        MockApiServices::default()
    }

    #[fixture]
    fn mock_app(
        mut mock_services: MockApiServices<MockGetProjector, MockCreateSubroutine>,
        mock_get_projector: MockGetProjector,
        mock_create_subroutine: MockCreateSubroutine,
    ) -> Router {
        mock_services
            .expect_projectors()
            .return_const(Arc::new(mock_get_projector));

        mock_services
            .expect_subroutines()
            .return_const(Arc::new(mock_create_subroutine));

        Router::new()
            .route("/:projector/", post(handler))
            .with_state(Arc::new(mock_services))
    }

    #[rstest]
    #[tokio::test]
    async fn returns_not_found_when_projector_does_not_exist(
        mock_services: MockApiServices<MockGetProjector, MockCreateSubroutine>,
        mut mock_get_projector: MockGetProjector,
        mock_create_subroutine: MockCreateSubroutine,
    ) {
        let get_projector_result = Err(ProjectorsError::NotFound("".into()));
        mock_get_projector
            .expect_get()
            .return_once(move |_| get_projector_result);

        let body = Body::from(
            serde_json::to_string(&NewSubroutine {
                subroutine_definition_id: "test".to_string(),
            })
            .unwrap(),
        );

        let response = mock_app(mock_services, mock_get_projector, mock_create_subroutine)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .uri("/nonexistent/")
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_conflict_when_subroutine_exists(
        mock_services: MockApiServices<MockGetProjector, MockCreateSubroutine>,
        mut mock_get_projector: MockGetProjector,
        mut mock_create_subroutine: MockCreateSubroutine,
        projector: ProjectorEntity,
    ) {
        let get_projector_result = Ok(projector.clone());
        mock_get_projector
            .expect_get()
            .return_once(move |_| get_projector_result);
        let create_subroutine_result = Err(SubroutinesError::AlreadyRunning);
        mock_create_subroutine
            .expect_create()
            .return_once(move |_| create_subroutine_result);

        let body = Body::from(
            serde_json::to_string(&NewSubroutine {
                subroutine_definition_id: "test".to_string(),
            })
            .unwrap(),
        );

        let response = mock_app(mock_services, mock_get_projector, mock_create_subroutine)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .uri(format!("/{}/", projector.id()))
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_created(
        mock_services: MockApiServices<MockGetProjector, MockCreateSubroutine>,
        mut mock_get_projector: MockGetProjector,
        mut mock_create_subroutine: MockCreateSubroutine,
        projector: ProjectorEntity,
        subroutine: SubroutineEntity,
    ) {
        let get_projector_result = Ok(projector.clone());
        mock_get_projector
            .expect_get()
            .return_once(move |_| get_projector_result);

        let create_subroutine_result = Ok(subroutine);
        mock_create_subroutine
            .expect_create()
            .return_once(move |_| create_subroutine_result);

        let body = Body::from(
            serde_json::to_string(&NewSubroutine {
                subroutine_definition_id: "test".to_string(),
            })
            .unwrap(),
        );

        let response = mock_app(mock_services, mock_get_projector, mock_create_subroutine)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .uri(format!("/{}/", projector.id()))
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_the_new_subroutine(
        mock_services: MockApiServices<MockGetProjector, MockCreateSubroutine>,
        mut mock_get_projector: MockGetProjector,
        mut mock_create_subroutine: MockCreateSubroutine,
        projector: ProjectorEntity,
        subroutine: SubroutineEntity,
    ) {
        let get_projector_result = Ok(projector.clone());
        mock_get_projector
            .expect_get()
            .return_once(move |_| get_projector_result);

        let create_subroutine_result = Ok(subroutine.clone());
        mock_create_subroutine
            .expect_create()
            .return_once(move |_| create_subroutine_result);

        let body = Body::from(
            serde_json::to_string(&NewSubroutine {
                subroutine_definition_id: "test".to_string(),
            })
            .unwrap(),
        );

        let response = mock_app(mock_services, mock_get_projector, mock_create_subroutine)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .uri(format!("/{}/", projector.id()))
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let p: SubroutineEntity = serde_json::from_slice(&body).unwrap();
        assert_eq!(p.id(), subroutine.id());
    }
}
