use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};

use crate::core::projectors::{
    entities::Projector,
    services::{FindProjectors, ProjectorsFindInput},
};

use super::ApiServices;

pub async fn handler<S>(
    State(state): State<Arc<ApiServices<S>>>,
) -> Result<Json<Vec<Projector>>, (StatusCode, String)>
where
    S: FindProjectors,
{
    let projectors = state
        .projectors()
        .find(ProjectorsFindInput::default())
        .await
        .unwrap();
    Ok(Json(projectors))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, routing::get, Router};
    use mockall::predicate::*;
    use rstest::*;
    use tower::ServiceExt;

    use crate::core::projectors::entities::fixtures::projector;
    use crate::core::projectors::services::MockFindProjectors;

    use super::*;

    #[fixture]
    fn mock_find() -> MockFindProjectors {
        MockFindProjectors::default()
    }

    #[fixture]
    fn mock_app(mock_find: MockFindProjectors) -> Router {
        let services = Arc::new(ApiServices {
            projectors_service: Arc::new(mock_find),
        });

        Router::new().route("/", get(handler)).with_state(services)
    }

    #[rstest]
    #[tokio::test]
    async fn gets_projectors(mut mock_find: MockFindProjectors) {
        mock_find
            .expect_find()
            .with(eq(ProjectorsFindInput::default()))
            .return_const(Ok(vec![]));

        mock_app(mock_find)
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_ok(mut mock_find: MockFindProjectors) {
        mock_find
            .expect_find()
            .with(eq(ProjectorsFindInput::default()))
            .return_const(Ok(vec![]));

        let response = mock_app(mock_find)
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_projectors(projector: Projector, mut mock_find: MockFindProjectors) {
        mock_find
            .expect_find()
            .with(eq(ProjectorsFindInput::default()))
            .return_const(Ok(vec![projector.clone()]));

        let response = mock_app(mock_find)
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let p: Vec<Projector> = serde_json::from_slice(&body).unwrap();
        assert_eq!(p.first().unwrap().id, projector.id);
    }
}
