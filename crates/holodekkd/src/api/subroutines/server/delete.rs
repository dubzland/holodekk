use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use holodekk::core::projectors::{GetProjector, ProjectorsGetInput};
use holodekk::core::subroutines::{DeleteSubroutine, SubroutinesDeleteInput};

use crate::api::ProjectorsApiServices;

use super::SubroutinesApiServices;

pub async fn handler<A, S, P>(
    State(state): State<Arc<A>>,
    Path((projector, subroutine)): Path<(String, String)>,
) -> Result<impl IntoResponse, crate::api::ApiError>
where
    A: SubroutinesApiServices<S> + ProjectorsApiServices<P>,
    S: DeleteSubroutine,
    P: GetProjector,
{
    state
        .projectors()
        .get(&ProjectorsGetInput::new(&projector))
        .await?;

    state
        .subroutines()
        .delete(&SubroutinesDeleteInput::new(&subroutine))
        .await?;
    Ok((StatusCode::NO_CONTENT, ""))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, routing::delete, Router};
    use mockall::mock;
    use rstest::*;
    use tower::ServiceExt;

    use holodekk::core::projectors::{entities::ProjectorEntity, ProjectorsError};
    use holodekk::core::subroutines::entities::SubroutineEntity;
    use holodekk::core::subroutines::SubroutinesError;

    use crate::api::fixtures::{
        mock_delete_subroutine, mock_get_projector, projector, subroutine, MockDeleteSubroutine,
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
    fn mock_services() -> MockApiServices<MockGetProjector, MockDeleteSubroutine> {
        MockApiServices::default()
    }

    #[fixture]
    fn mock_app(
        mut mock_services: MockApiServices<MockGetProjector, MockDeleteSubroutine>,
        mock_get_projector: MockGetProjector,
        mock_delete_subroutine: MockDeleteSubroutine,
    ) -> Router {
        mock_services
            .expect_projectors()
            .return_const(Arc::new(mock_get_projector));

        mock_services
            .expect_subroutines()
            .return_const(Arc::new(mock_delete_subroutine));

        Router::new()
            .route("/:projector/subroutines/:subroutine", delete(handler))
            .with_state(Arc::new(mock_services))
    }

    #[rstest]
    #[tokio::test]
    async fn returns_not_found_when_projector_does_not_exist(
        mock_services: MockApiServices<MockGetProjector, MockDeleteSubroutine>,
        mut mock_get_projector: MockGetProjector,
        mock_delete_subroutine: MockDeleteSubroutine,
        subroutine: SubroutineEntity,
    ) {
        let get_projector_result = Err(ProjectorsError::NotFound("nonexistent".into()));
        mock_get_projector
            .expect_get()
            .return_once(move |_| get_projector_result);

        let response = mock_app(mock_services, mock_get_projector, mock_delete_subroutine)
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/nonexistent/subroutines/{}", subroutine.id()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_not_found_when_subroutine_does_not_exist(
        mock_services: MockApiServices<MockGetProjector, MockDeleteSubroutine>,
        mut mock_get_projector: MockGetProjector,
        mut mock_delete_subroutine: MockDeleteSubroutine,
        projector: ProjectorEntity,
        subroutine: SubroutineEntity,
    ) {
        let get_projector_result = Ok(projector);
        mock_get_projector
            .expect_get()
            .return_once(move |_| get_projector_result);
        let delete_subroutine_result = Err(SubroutinesError::NotFound(subroutine.id().into()));
        mock_delete_subroutine
            .expect_delete()
            .return_once(move |_| delete_subroutine_result);

        let response = mock_app(mock_services, mock_get_projector, mock_delete_subroutine)
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/nonexistent/subroutines/{}", subroutine.id()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_no_content(
        mock_services: MockApiServices<MockGetProjector, MockDeleteSubroutine>,
        mut mock_get_projector: MockGetProjector,
        mut mock_delete_subroutine: MockDeleteSubroutine,
        projector: ProjectorEntity,
        subroutine: SubroutineEntity,
    ) {
        let get_projector_result = Ok(projector.clone());
        mock_get_projector
            .expect_get()
            .return_once(move |_| get_projector_result);

        let delete_subroutine_result = Ok(());
        mock_delete_subroutine
            .expect_delete()
            .return_once(move |_| delete_subroutine_result);

        let response = mock_app(mock_services, mock_get_projector, mock_delete_subroutine)
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!(
                        "/{}/subroutines/{}",
                        projector.id(),
                        subroutine.id()
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}
