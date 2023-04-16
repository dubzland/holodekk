use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::core::projectors::{
    api::models::NewProjector,
    services::{CreateProjector, ProjectorsCreateInput},
};

use super::ApiServices;

pub async fn handler<P, D>(
    State(state): State<Arc<ApiServices<P, D>>>,
    Json(new_projector): Json<NewProjector>,
) -> Result<impl IntoResponse, crate::core::services::Error>
where
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

    use crate::core::projectors::entities::{fixtures::projector, Projector};
    use crate::core::projectors::services::MockCreateProjector;
    use crate::core::services::Error;
    use crate::core::subroutine_definitions::services::MockCreateSubroutineDefinition;

    use super::*;

    #[fixture]
    fn mock_service() -> MockCreateProjector {
        MockCreateProjector::default()
    }

    #[fixture]
    fn mock_app(mock_service: MockCreateProjector) -> Router {
        let services = Arc::new(ApiServices {
            projectors_service: Arc::new(mock_service),
            definitions_service: Arc::new(MockCreateSubroutineDefinition::default()),
        });

        Router::new().route("/", post(handler)).with_state(services)
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_conflict_when_projector_exists(mut mock_service: MockCreateProjector) {
        mock_service
            .expect_create()
            .withf(|input| input.namespace() == "test")
            .return_const(Err(Error::Duplicate));

        let body = Body::from(
            serde_json::to_string(&NewProjector {
                namespace: "test".to_string(),
            })
            .unwrap(),
        );

        let response = mock_app(mock_service)
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
    async fn responds_with_created(mut mock_service: MockCreateProjector, projector: Projector) {
        mock_service
            .expect_create()
            .withf(|input| input.namespace() == "test")
            .return_const(Ok(projector));

        let body = Body::from(
            serde_json::to_string(&NewProjector {
                namespace: "test".to_string(),
            })
            .unwrap(),
        );

        let response = mock_app(mock_service)
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
        mut mock_service: MockCreateProjector,
        projector: Projector,
    ) {
        mock_service
            .expect_create()
            .withf(|input| input.namespace() == "test")
            .return_const(Ok(projector.clone()));

        let body = Body::from(
            serde_json::to_string(&NewProjector {
                namespace: "test".to_string(),
            })
            .unwrap(),
        );

        let response = mock_app(mock_service)
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
