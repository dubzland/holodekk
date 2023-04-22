use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use holodekk::core::projectors::{CreateProjector, ProjectorsCreateInput};

use crate::api::projectors::models::NewProjector;
use crate::api::ApiError;

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
    use async_trait::async_trait;
    use axum::{body::Body, http::Request, routing::post, Router};
    use mockall::mock;
    use rstest::*;
    use tower::ServiceExt;

    use holodekk::core::projectors::entities::ProjectorEntity;
    use holodekk::core::projectors::{ProjectorsError, Result};

    use crate::api::projectors::server::MockProjectorsApiServices;

    use super::*;

    mock! {
        pub ProjectorsService {}
        #[async_trait]
        impl CreateProjector for ProjectorsService {
            async fn create<'a>(&self, input: &'a ProjectorsCreateInput<'a>) -> Result<ProjectorEntity>;
        }
    }

    #[fixture]
    fn projector() -> ProjectorEntity {
        ProjectorEntity::new("test", "/tmp/projector")
    }

    #[fixture]
    fn mock_services() -> MockProjectorsApiServices<MockProjectorsService> {
        MockProjectorsApiServices::default()
    }

    #[fixture]
    fn mock_create() -> MockProjectorsService {
        MockProjectorsService::default()
    }

    #[fixture]
    fn mock_app(
        mut mock_services: MockProjectorsApiServices<MockProjectorsService>,
        mock_create: MockProjectorsService,
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
        mock_services: MockProjectorsApiServices<MockProjectorsService>,
        mut mock_create: MockProjectorsService,
    ) {
        let create_result = Err(ProjectorsError::AlreadyRunning("test".into()));
        mock_create
            .expect_create()
            .withf(|input| input.namespace() == "test")
            .return_once(move |_| create_result);

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
        mock_services: MockProjectorsApiServices<MockProjectorsService>,

        mut mock_create: MockProjectorsService,
        projector: ProjectorEntity,
    ) {
        let create_result = Ok(projector.clone());
        mock_create
            .expect_create()
            .withf(|input| input.namespace() == "test")
            .return_once(move |_| create_result);

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
        mock_services: MockProjectorsApiServices<MockProjectorsService>,

        mut mock_create: MockProjectorsService,
        projector: ProjectorEntity,
    ) {
        let create_result = Ok(projector.clone());
        mock_create
            .expect_create()
            .withf(|input| input.namespace() == "test")
            .return_once(move |_| create_result);

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
        let p: ProjectorEntity = serde_json::from_slice(&body).unwrap();
        assert_eq!(p.id(), projector.id());
    }
}
