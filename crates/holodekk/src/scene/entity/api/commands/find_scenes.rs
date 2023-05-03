use std::sync::Arc;

use axum::extract::State;

use crate::apis::http::{ApiState, GetResponse};
use crate::entity;
use crate::scene::entity::{
    api::models::Scene,
    service::{find::Input, Find},
};

pub async fn find_scenes<A, E, U>(
    State(state): State<Arc<A>>,
) -> Result<GetResponse<Vec<Scene>>, entity::service::Error>
where
    A: ApiState<E, U>,
    E: Find,
    U: Send + Sync + 'static,
{
    let scenes = state.scene_entity_service().find(&Input::default()).await?;

    Ok(GetResponse(scenes.into_iter().map(Into::into).collect()))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use rstest::*;
    use tower::ServiceExt;

    use crate::apis::http::MockApiState;
    use crate::scene::{
        self,
        entity::{
            mock_entity,
            service::{mock_find, MockFind},
        },
    };
    use crate::subroutine::entity::service::MockService as MockSubroutineService;

    use super::*;

    fn mock_app(mock_find: MockFind) -> Router {
        let mut state = MockApiState::<MockFind, MockSubroutineService>::default();
        state
            .expect_scene_entity_service()
            .return_once(move || Arc::new(mock_find));
        Router::new()
            .route("/", get(find_scenes))
            .with_state(Arc::new(state))
    }

    fn make_request(
        mock_find: MockFind,
    ) -> tower::util::Oneshot<axum::Router, http::Request<hyper::Body>> {
        mock_app(mock_find).oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
    }

    #[rstest]
    #[tokio::test]
    async fn gets_scenes_from_service(mut mock_find: MockFind) {
        mock_find.expect_find().return_once(move |_| Ok(vec![]));

        make_request(mock_find).await.unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn responds_with_ok(mut mock_find: MockFind) {
        mock_find.expect_find().return_once(move |_| Ok(vec![]));

        let response = make_request(mock_find).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[rstest]
    #[tokio::test]
    async fn returns_scenes(mut mock_find: MockFind, mock_entity: scene::Entity) {
        {
            let entities = vec![mock_entity.clone()];
            mock_find.expect_find().return_once(move |_| Ok(entities));
        }

        let response = make_request(mock_find).await.unwrap();

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let p: Vec<scene::Entity> = serde_json::from_slice(&body).unwrap();
        assert_eq!(p.first().unwrap(), &mock_entity);
    }
}
