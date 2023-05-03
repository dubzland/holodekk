use async_trait::async_trait;
use log::trace;
#[cfg(test)]
use mockall::automock;

use crate::entity;
use crate::scene;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Input<'f> {
    pub name: Option<&'f str>,
}

impl<'f> Input<'f> {
    pub fn new(name: Option<&'f str>) -> Self {
        Self { name }
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Find: Send + Sync + 'static {
    async fn find<'a>(&self, input: &'a Input<'a>) -> entity::service::Result<Vec<scene::Entity>>;
}

#[async_trait]
impl<R> Find for super::Service<R>
where
    R: scene::entity::Repository,
{
    async fn find<'a>(&self, input: &'a Input<'a>) -> entity::service::Result<Vec<scene::Entity>> {
        trace!("scene::entity::Service#delete({:?})", input);

        let query = scene::entity::repository::Query::default();

        let scenes = self.repo.scenes_find(query).await?;
        Ok(scenes)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::entity;
    use crate::scene::{
        self,
        entity::{
            mock_entity,
            repository::{mock_repository, MockRepository},
        },
    };

    use super::*;

    async fn execute(repo: MockRepository) -> entity::service::Result<Vec<scene::Entity>> {
        let service = scene::entity::Service::new(Arc::new(repo));

        service.find(&super::Input::default()).await
    }

    #[rstest]
    #[tokio::test]
    async fn executes_query(mut mock_repository: MockRepository) {
        mock_repository
            .expect_scenes_find()
            .withf(|query: &scene::entity::repository::Query| {
                query == &scene::entity::repository::Query::default()
            })
            .return_once(move |_| Ok(vec![]));

        execute(mock_repository).await.unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_results_of_query(
        mut mock_repository: MockRepository,
        mock_entity: scene::Entity,
    ) {
        {
            let entities = vec![mock_entity.clone()];
            mock_repository
                .expect_scenes_find()
                .return_once(move |_| Ok(entities));
        }

        let scenes = execute(mock_repository).await.unwrap();
        assert_eq!(scenes, vec![mock_entity]);
    }
}
