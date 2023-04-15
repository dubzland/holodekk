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
}
