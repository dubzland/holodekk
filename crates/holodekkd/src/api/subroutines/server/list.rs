use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};

use holodekk::core::projectors::{GetProjector, ProjectorsGetInput};
use holodekk::core::subroutines::{FindSubroutines, SubroutinesFindInput};

use crate::api::ProjectorsApiServices;

use super::SubroutinesApiServices;

pub async fn handler<A, S, P>(
    State(state): State<Arc<A>>,
    Path(projector): Path<String>,
) -> Result<impl IntoResponse, crate::api::ApiError>
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
        .find(&SubroutinesFindInput::new(Some(projector.id()), None))
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

    use holodekk::core::projectors::{entities::ProjectorEntity, ProjectorsError};
    use holodekk::core::subroutines::entities::SubroutineEntity;
    // use holodekk::core::subroutines::MockFindSubroutines;

    use crate::api::fixtures::{
        mock_find_subroutines, mock_get_projector, projector, subroutine, MockFindSubroutines,
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
        let get_projector_result = Err(ProjectorsError::NotFound("nonexistent".into()));
        mock_get_projector
            .expect_get()
            .return_once(move |_| get_projector_result);

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
        projector: ProjectorEntity,
    ) {
        let get_projector_result = Ok(projector.clone());
        mock_get_projector
            .expect_get()
            .return_once(move |_| get_projector_result);
        let find_subroutines_result = Ok(vec![]);
        mock_find_subroutines
            .expect_find()
            .return_once(move |_| find_subroutines_result);

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
        projector: ProjectorEntity,
        subroutine: SubroutineEntity,
    ) {
        let get_projector_result = Ok(projector.clone());
        mock_get_projector
            .expect_get()
            .return_once(move |_| get_projector_result);
        let find_subroutines_result = Ok(vec![subroutine.clone()]);
        mock_find_subroutines
            .expect_find()
            .return_once(move |_| find_subroutines_result);

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
        let p: Vec<SubroutineEntity> = serde_json::from_slice(&body).unwrap();
        assert_eq!(p.first().unwrap().id(), subroutine.id());
    }
}
