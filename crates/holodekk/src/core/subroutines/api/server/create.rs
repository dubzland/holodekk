use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::core::projectors::{
    api::server::ProjectorsApiServices, GetProjector, ProjectorsGetInput,
};
use crate::core::subroutines::{
    api::models::NewSubroutine, CreateSubroutine, SubroutinesCreateInput,
};

use super::SubroutinesApiServices;

pub async fn handler<A, S, P>(
    State(state): State<Arc<A>>,
    Path(projector): Path<String>,
    Json(new_subroutine): Json<NewSubroutine>,
) -> Result<impl IntoResponse, crate::core::api::ApiError>
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
            projector.fleet(),
            projector.namespace(),
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

    use crate::core::projectors::{
        entities::{fixtures::projector, Projector},
        fixtures::mock_get_projector,
        MockGetProjector, ProjectorsError,
    };
    use crate::core::repositories::RepositoryId;
    use crate::core::subroutines::entities::{fixtures::subroutine, Subroutine};
    use crate::core::subroutines::{
        fixtures::mock_create_subroutine, MockCreateSubroutine, SubroutinesError,
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
        mock_get_projector
            .expect_get()
            .return_const(Err(ProjectorsError::NotFound("".into())));

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
        projector: Projector,
    ) {
        mock_get_projector
            .expect_get()
            .return_const(Ok(projector.clone()));
        mock_create_subroutine
            .expect_create()
            .withf(|input| input.namespace() == "test")
            .return_const(Err(SubroutinesError::AlreadyRunning("test".into())));

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
        projector: Projector,
        subroutine: Subroutine,
    ) {
        mock_get_projector
            .expect_get()
            .return_const(Ok(projector.clone()));

        mock_create_subroutine
            .expect_create()
            .withf(|input| input.namespace() == "test")
            .return_const(Ok(subroutine));

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
        projector: Projector,
        subroutine: Subroutine,
    ) {
        mock_get_projector
            .expect_get()
            .return_const(Ok(projector.clone()));

        mock_create_subroutine
            .expect_create()
            .withf(|input| input.namespace() == "test")
            .return_const(Ok(subroutine.clone()));

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
        let p: Subroutine = serde_json::from_slice(&body).unwrap();
        assert_eq!(p.id(), subroutine.id());
    }
}
