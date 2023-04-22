use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};

use holodekk::core::projectors::{FindProjectors, ProjectorsFindInput};

use super::ProjectorsApiServices;

pub async fn handler<S, P>(
    State(state): State<Arc<S>>,
) -> Result<impl IntoResponse, crate::api::ApiError>
where
    S: ProjectorsApiServices<P>,
    P: FindProjectors,
{
    let projectors = state
        .projectors()
        .find(&ProjectorsFindInput::default())
        .await?;
    Ok(Json(projectors))
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use mockall::mock;
    use rstest::*;
    use tower::ServiceExt;

    use holodekk::core::projectors::{entities::ProjectorEntity, Result};

    use crate::api::projectors::server::MockProjectorsApiServices;

    use super::*;

    mock! {
        pub ProjectorsService {}
        #[async_trait]
        impl FindProjectors for ProjectorsService {
            async fn find<'a>(&self, input: &'a ProjectorsFindInput<'a>) -> Result<Vec<ProjectorEntity>>;
        }
    }

    #[fixture]
    fn projector() -> ProjectorEntity {
        ProjectorEntity::new("test", "/tmp/projector")
    }

    #[fixture]
    fn mock_find() -> MockProjectorsService {
        MockProjectorsService::default()
    }

    #[fixture]
    fn mock_services() -> MockProjectorsApiServices<MockProjectorsService> {
        MockProjectorsApiServices::default()
    }

    #[fixture]
    fn mock_app(
        mut mock_services: MockProjectorsApiServices<MockProjectorsService>,
        mock_find: MockProjectorsService,
    ) -> Router {
        mock_services
            .expect_projectors()
            .return_const(Arc::new(mock_find));

        Router::new()
            .route("/", get(handler))
            .with_state(Arc::new(mock_services))
    }

    #[rstest]
    #[tokio::test]
    async fn gets_projectors(
        mock_services: MockProjectorsApiServices<MockProjectorsService>,
        mut mock_find: MockProjectorsService,
    ) {
        let input = ProjectorsFindInput::default();

        let find_result = Ok(vec![]);
        mock_find
            .expect_find()
            .withf(move |i| i == &input)
            .return_once(move |_| find_result);

        mock_app(mock_services, mock_find)
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_ok(
        mock_services: MockProjectorsApiServices<MockProjectorsService>,
        mut mock_find: MockProjectorsService,
    ) {
        let input = ProjectorsFindInput::default();

        let find_result = Ok(vec![]);
        mock_find
            .expect_find()
            .withf(move |i| i == &input)
            .return_once(move |_| find_result);

        let response = mock_app(mock_services, mock_find)
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_projectors(
        mock_services: MockProjectorsApiServices<MockProjectorsService>,
        projector: ProjectorEntity,
        mut mock_find: MockProjectorsService,
    ) {
        let input = ProjectorsFindInput::default();

        let find_result = Ok(vec![projector.clone()]);
        mock_find
            .expect_find()
            .withf(move |i| i == &input)
            .return_once(move |_| find_result);

        let response = mock_app(mock_services, mock_find)
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let p: Vec<ProjectorEntity> = serde_json::from_slice(&body).unwrap();
        assert_eq!(p.first().unwrap().id(), projector.id());
    }
}
