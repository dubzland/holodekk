use async_trait::async_trait;
use log::trace;
#[cfg(test)]
use mockall::automock;

use crate::entity::service::Result;
use crate::images::SubroutineImageId;
use crate::subroutine::{
    entity::{repository::Query, Id, Repository},
    Entity,
};

use super::Service;

#[derive(Clone, Debug, PartialEq)]
pub struct Input<'f> {
    pub scene_entity_id: Option<&'f str>,
    pub subroutine_image_id: Option<&'f str>,
}

impl<'f> Input<'f> {
    pub fn new(scene_entity_id: Option<&'f str>, subroutine_image_id: Option<&'f str>) -> Self {
        Self {
            scene_entity_id,
            subroutine_image_id,
        }
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Find: Send + Sync + 'static {
    async fn find<'a>(&self, input: &'a Input<'a>) -> Result<Vec<Entity>>;
}

#[async_trait]
impl<R> Find for Service<R>
where
    R: Repository,
{
    async fn find<'a>(&self, input: &'a Input<'a>) -> Result<Vec<Entity>> {
        trace!("subroutine::entity::Service::find({:?})", input);

        let mut query = Query::builder();
        let scene_id: Id;
        let image_id: SubroutineImageId;
        if let Some(scene_entity_id) = input.scene_entity_id {
            scene_id = scene_entity_id.parse()?;
            query.for_scene_entity(&scene_id);
        }
        if let Some(subroutine_image_id) = input.subroutine_image_id {
            image_id = subroutine_image_id.parse()?;
            query.for_subroutine_image(&image_id);
        }

        let subroutines = self.repo.subroutines_find(query).await?;
        Ok(subroutines)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::images::ImageName;
    use crate::scene;
    use crate::subroutine::entity::{
        mock_entity,
        repository::{mock_repository, MockRepository},
    };

    use super::*;

    async fn execute(repo: MockRepository, scene: &str, image: &str) -> Result<Vec<Entity>> {
        let service = Service::new(Arc::new(repo));

        service.find(&Input::new(Some(scene), Some(image))).await
    }

    #[rstest]
    #[tokio::test]
    async fn executes_query(mut mock_repository: MockRepository) {
        let scene_entity_id = scene::entity::Id::generate();
        let subroutine_image_id = SubroutineImageId::generate(&ImageName::from("test"));

        {
            let scene_entity_id = scene_entity_id.clone();
            let subroutine_image_id = subroutine_image_id.clone();
            mock_repository
                .expect_subroutines_find()
                .withf(move |query: &Query| {
                    query.scene_entity_id == Some(&scene_entity_id)
                        && query.subroutine_image_id == Some(&subroutine_image_id)
                })
                .return_once(|_| Ok(vec![]));
        }

        execute(mock_repository, &scene_entity_id, &subroutine_image_id)
            .await
            .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_results_of_query(mut mock_repository: MockRepository, mock_entity: Entity) {
        let scene_entity_id = mock_entity.scene_entity_id.clone();
        let subroutine_image_id = mock_entity.subroutine_image_id.clone();

        {
            let mock_entity = mock_entity.clone();
            mock_repository
                .expect_subroutines_find()
                .return_once(move |_| Ok(vec![mock_entity]));
        }

        let subroutines = execute(mock_repository, &scene_entity_id, &subroutine_image_id)
            .await
            .unwrap();
        assert_eq!(subroutines, vec![mock_entity]);
    }
}
