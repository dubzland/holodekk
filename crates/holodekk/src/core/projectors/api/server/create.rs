use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::core::api::ApiError;
use crate::core::projectors::{api::models::NewProjector, CreateProjector, ProjectorsCreateInput};

use super::ProjectorsApiServices;

pub async fn handler<S, P>(
    State(state): State<Arc<S>>,
    Json(new_projector): Json<NewProjector>,
) -> Result<impl IntoResponse, ApiError>
where
    S: ProjectorsApiServices<P>,
    P: CreateProjector,
{
    let projector = state
        .projectors()
        .create(&ProjectorsCreateInput::new(&new_projector.namespace))
        .await?;
    Ok((StatusCode::CREATED, Json(projector)))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, routing::post, Router};
    use rstest::*;
    use tower::ServiceExt;

    use crate::core::projectors::api::server::MockProjectorsApiServices;
    use crate::core::projectors::entities::{fixtures::projector, Projector};
    use crate::core::projectors::{MockCreateProjector, ProjectorsError};

    use super::*;

    #[fixture]
    fn mock_services() -> MockProjectorsApiServices<MockCreateProjector> {
        MockProjectorsApiServices::default()
    }

    #[fixture]
    fn mock_create() -> MockCreateProjector {
        MockCreateProjector::default()
    }

    #[fixture]
    fn mock_app(
        mut mock_services: MockProjectorsApiServices<MockCreateProjector>,
        mock_create: MockCreateProjector,
    ) -> Router {
        mock_services
            .expect_projectors()
            .return_const(Arc::new(mock_create));

        Router::new()
            .route("/", post(handler))
            .with_state(Arc::new(mock_services))
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_conflict_when_projector_exists(
        mock_services: MockProjectorsApiServices<MockCreateProjector>,

        mut mock_create: MockCreateProjector,
    ) {
        mock_create
            .expect_create()
            .withf(|input| input.namespace() == "test")
            .return_const(Err(ProjectorsError::AlreadyRunning("test".into())));

        let body = Body::from(
            serde_json::to_string(&NewProjector {
                namespace: "test".to_string(),
            })
            .unwrap(),
        );

        let response = mock_app(mock_services, mock_create)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .uri("/")
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
        mock_services: MockProjectorsApiServices<MockCreateProjector>,

        mut mock_create: MockCreateProjector,
        projector: Projector,
    ) {
        mock_create
            .expect_create()
            .withf(|input| input.namespace() == "test")
            .return_const(Ok(projector));

        let body = Body::from(
            serde_json::to_string(&NewProjector {
                namespace: "test".to_string(),
            })
            .unwrap(),
        );

        let response = mock_app(mock_services, mock_create)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .uri("/")
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_the_new_projector(
        mock_services: MockProjectorsApiServices<MockCreateProjector>,

        mut mock_create: MockCreateProjector,
        projector: Projector,
    ) {
        mock_create
            .expect_create()
            .withf(|input| input.namespace() == "test")
            .return_const(Ok(projector.clone()));

        let body = Body::from(
            serde_json::to_string(&NewProjector {
                namespace: "test".to_string(),
            })
            .unwrap(),
        );

        let response = mock_app(mock_services, mock_create)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .uri("/")
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let p: Projector = serde_json::from_slice(&body).unwrap();
        assert_eq!(p.id(), projector.id());
    }
}
