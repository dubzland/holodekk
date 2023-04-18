use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};

use crate::core::projectors::{
    api::server::ProjectorsApiServices, GetProjector, ProjectorsGetInput,
};
use crate::core::subroutines::{FindSubroutines, SubroutinesFindInput};

use super::SubroutinesApiServices;

pub async fn handler<A, S, P>(
    State(state): State<Arc<A>>,
    Path(projector): Path<String>,
) -> Result<impl IntoResponse, crate::core::api::ApiError>
where
    A: SubroutinesApiServices<S> + ProjectorsApiServices<P>,
    S: FindSubroutines,
    P: GetProjector,
{
    let projector = state
        .projectors()
        .get(&ProjectorsGetInput::new(&projector))
        .await?;

    let subroutines = state
        .subroutines()
        .find(&SubroutinesFindInput::new(
            None,
            Some(projector.namespace()),
            None,
        ))
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
    use mockall::mock;
    use rstest::*;
    use tower::ServiceExt;

    use crate::core::projectors::{
        entities::{fixtures::projector, Projector},
        MockGetProjector, ProjectorsError,
    };
    use crate::core::repositories::RepositoryId;
    use crate::core::subroutines::entities::{fixtures::subroutine, Subroutine};
    use crate::core::subroutines::MockFindSubroutines;

    use super::*;

    #[fixture]
    fn mock_find_subroutines() -> MockFindSubroutines {
        MockFindSubroutines::default()
    }

    #[fixture]
    fn mock_get_projector() -> MockGetProjector {
        MockGetProjector::default()
    }

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
    fn mock_services() -> MockApiServices<MockGetProjector, MockFindSubroutines> {
        MockApiServices::default()
    }

    #[fixture]
    fn mock_app(
        mut mock_services: MockApiServices<MockGetProjector, MockFindSubroutines>,
        mock_get_projector: MockGetProjector,
        mock_find_subroutines: MockFindSubroutines,
    ) -> Router {
        mock_services
            .expect_projectors()
            .return_const(Arc::new(mock_get_projector));

        mock_services
            .expect_subroutines()
            .return_const(Arc::new(mock_find_subroutines));

        Router::new()
            .route("/:projector/", get(handler))
            .with_state(Arc::new(mock_services))
    }

    #[rstest]
    #[tokio::test]
    async fn returns_not_found_when_projector_does_not_exist(
        mock_services: MockApiServices<MockGetProjector, MockFindSubroutines>,
        mut mock_get_projector: MockGetProjector,
        mock_find_subroutines: MockFindSubroutines,
    ) {
        mock_get_projector
            .expect_get()
            .return_const(Err(ProjectorsError::NotFound("".into())));

        let response = mock_app(mock_services, mock_get_projector, mock_find_subroutines)
            .oneshot(
                Request::builder()
                    .uri(format!("/nonexistent/"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_ok(
        mock_services: MockApiServices<MockGetProjector, MockFindSubroutines>,
        mut mock_get_projector: MockGetProjector,
        mut mock_find_subroutines: MockFindSubroutines,
        projector: Projector,
    ) {
        mock_get_projector
            .expect_get()
            .return_const(Ok(projector.clone()));
        mock_find_subroutines.expect_find().return_const(Ok(vec![]));

        let response = mock_app(mock_services, mock_get_projector, mock_find_subroutines)
            .oneshot(
                Request::builder()
                    .uri(format!("/{}/", projector.id()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_subroutines(
        mock_services: MockApiServices<MockGetProjector, MockFindSubroutines>,
        mut mock_get_projector: MockGetProjector,
        mut mock_find_subroutines: MockFindSubroutines,
        projector: Projector,
        subroutine: Subroutine,
    ) {
        mock_get_projector
            .expect_get()
            .return_const(Ok(projector.clone()));
        mock_find_subroutines
            .expect_find()
            .return_const(Ok(vec![subroutine.clone()]));

        let response = mock_app(mock_services, mock_get_projector, mock_find_subroutines)
            .oneshot(
                Request::builder()
                    .uri(format!("/{}/", projector.id()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let p: Vec<Subroutine> = serde_json::from_slice(&body).unwrap();
        assert_eq!(p.first().unwrap().id(), subroutine.id());
    }
}
