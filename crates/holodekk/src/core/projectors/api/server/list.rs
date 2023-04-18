use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};

use crate::core::projectors::{entities::Projector, FindProjectors, ProjectorsFindInput};

use super::ProjectorApiServices;

pub async fn handler<S, P>(
    State(state): State<Arc<S>>,
) -> Result<Json<Vec<Projector>>, (StatusCode, String)>
where
    S: ProjectorApiServices<P>,
    P: FindProjectors,
{
    let projectors = state
        .projectors()
        .find(&ProjectorsFindInput::default())
        .await;
    Ok(Json(projectors))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, routing::get, Router};
    use rstest::*;
    use tower::ServiceExt;

    use crate::core::projectors::api::server::MockProjectorApiServices;
    use crate::core::projectors::entities::fixtures::projector;
    use crate::core::projectors::MockFindProjectors;

    use super::*;

    #[fixture]
    fn mock_find() -> MockFindProjectors {
        MockFindProjectors::default()
    }

    #[fixture]
    fn mock_services() -> MockProjectorApiServices<MockFindProjectors> {
        MockProjectorApiServices::default()
    }

    #[fixture]
    fn mock_app(
        mut mock_services: MockProjectorApiServices<MockFindProjectors>,
        mock_find: MockFindProjectors,
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
        mock_services: MockProjectorApiServices<MockFindProjectors>,
        mut mock_find: MockFindProjectors,
    ) {
        let input = ProjectorsFindInput::default();
        mock_find
            .expect_find()
            .withf(move |i| i == &input)
            .return_const(vec![]);

        mock_app(mock_services, mock_find)
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_ok(
        mock_services: MockProjectorApiServices<MockFindProjectors>,
        mut mock_find: MockFindProjectors,
    ) {
        let input = ProjectorsFindInput::default();

        mock_find
            .expect_find()
            .withf(move |i| i == &input)
            .return_const(vec![]);

        let response = mock_app(mock_services, mock_find)
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_projectors(
        mock_services: MockProjectorApiServices<MockFindProjectors>,
        projector: Projector,
        mut mock_find: MockFindProjectors,
    ) {
        let input = ProjectorsFindInput::default();

        mock_find
            .expect_find()
            .withf(move |i| i == &input)
            .return_const(vec![projector.clone()]);

        let response = mock_app(mock_services, mock_find)
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let p: Vec<Projector> = serde_json::from_slice(&body).unwrap();
        assert_eq!(p.first().unwrap().id(), projector.id());
    }
}
