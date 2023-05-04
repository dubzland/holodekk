use async_trait::async_trait;
use log::trace;
#[cfg(test)]
use mockall::automock;

use crate::entity::service::Result;
use crate::subroutine::{
    entity::{repository::Query, Id, Repository},
    image, Entity,
};

use super::Service;

/// Input requirements for [`Find::find()`]
#[derive(Clone, Debug, PartialEq)]
pub struct Input<'f> {
    /// [`Id`][`crate::scene::entity::Id`] of the `scene` entity to which this subroutine instance
    /// belongs
    pub scene_entity_id: Option<&'f str>,
    /// [`Id`][`image::Id`] of the `image` this subroutine instance is running
    pub image_id: Option<&'f str>,
}

impl<'f> Input<'f> {
    /// Shorthand for instanciating a new [`Input`] instance
    #[must_use]
    pub fn new(scene_entity_id: Option<&'f str>, image_id: Option<&'f str>) -> Self {
        Self {
            scene_entity_id,
            image_id,
        }
    }
}

/// Retrieve one or more [`Entities`][`Entity`] from the [`Repository`]
#[cfg_attr(test, automock)]
#[async_trait]
pub trait Find: Send + Sync + 'static {
    /// Retrieves subroutine instances from the repository based on the input data.
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
        let image_id: image::Id;
        if let Some(id) = input.scene_entity_id {
            scene_id = id.parse()?;
            query.for_scene_entity(&scene_id);
        }
        if let Some(id) = input.image_id {
            image_id = id.parse()?;
            query.for_image(&image_id);
        }

        let subroutines = self.repo.subroutines_find(query).await?;
        Ok(subroutines)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::image;
    use crate::scene;
    use crate::subroutine::{
        self,
        entity::{
            mock_entity,
            repository::{mock_repository, MockRepository},
        },
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
        let image_id = subroutine::image::Id::generate(&image::Name::from("test"));

        {
            let scene_entity_id = scene_entity_id.clone();
            let image_id = image_id.clone();
            mock_repository
                .expect_subroutines_find()
                .withf(move |query: &Query| {
                    query.scene_entity_id == Some(&scene_entity_id)
                        && query.image_id == Some(&image_id)
                })
                .return_once(|_| Ok(vec![]));
        }

        execute(mock_repository, &scene_entity_id, &image_id)
            .await
            .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_results_of_query(mut mock_repository: MockRepository, mock_entity: Entity) {
        let scene_entity_id = mock_entity.scene_entity_id.clone();
        let image_id = mock_entity.image_id.clone();

        {
            let mock_entity = mock_entity.clone();
            mock_repository
                .expect_subroutines_find()
                .return_once(move |_| Ok(vec![mock_entity]));
        }

        let subroutines = execute(mock_repository, &scene_entity_id, &image_id)
            .await
            .unwrap();
        assert_eq!(subroutines, vec![mock_entity]);
    }
}
