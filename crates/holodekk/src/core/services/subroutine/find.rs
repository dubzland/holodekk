use async_trait::async_trait;
use log::trace;

use crate::core::{
    entities::{SceneEntityId, SubroutineEntity, SubroutinesQuery, SubroutinesRepository},
    images::SubroutineImageId,
};

use super::{FindSubroutines, Result, SubroutinesFindInput, SubroutinesService};

#[async_trait]
impl<R> FindSubroutines for SubroutinesService<R>
where
    R: SubroutinesRepository,
{
    async fn find<'a>(&self, input: &'a SubroutinesFindInput<'a>) -> Result<Vec<SubroutineEntity>> {
        trace!("SubroutinesService::find({:?})", input);

        let mut query = SubroutinesQuery::builder();
        let scene_id: SceneEntityId;
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

    use crate::core::{
        entities::{
            fixtures::{mock_subroutine_entity, mock_subroutines_repository},
            MockSubroutinesRepository, SubroutineEntity,
        },
        images::ImageName,
    };
    use rstest::*;

    use super::*;

    async fn execute(
        repo: MockSubroutinesRepository,
        scene: &str,
        image: &str,
    ) -> Result<Vec<SubroutineEntity>> {
        let service = SubroutinesService::new(Arc::new(repo));

        service
            .find(&SubroutinesFindInput::new(Some(scene), Some(image)))
            .await
    }

    #[rstest]
    #[tokio::test]
    async fn executes_query(mut mock_subroutines_repository: MockSubroutinesRepository) {
        let scene_entity_id = SceneEntityId::generate();
        let subroutine_image_id = SubroutineImageId::generate(&ImageName::from("test"));

        {
            let scene_entity_id = scene_entity_id.clone();
            let subroutine_image_id = subroutine_image_id.clone();
            mock_subroutines_repository
                .expect_subroutines_find()
                .withf(move |query: &SubroutinesQuery| {
                    query.scene_entity_id == Some(&scene_entity_id)
                        && query.subroutine_image_id == Some(&subroutine_image_id)
                })
                .return_once(|_| Ok(vec![]));
        }

        execute(
            mock_subroutines_repository,
            &scene_entity_id,
            &subroutine_image_id,
        )
        .await
        .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_results_of_query(
        mut mock_subroutines_repository: MockSubroutinesRepository,
        mock_subroutine_entity: SubroutineEntity,
    ) {
        let scene_entity_id = mock_subroutine_entity.scene_entity_id.clone();
        let subroutine_image_id = mock_subroutine_entity.subroutine_image_id.clone();

        {
            let mock_subroutine_entity = mock_subroutine_entity.clone();
            mock_subroutines_repository
                .expect_subroutines_find()
                .return_once(move |_| Ok(vec![mock_subroutine_entity]));
        }

        let subroutines = execute(
            mock_subroutines_repository,
            &scene_entity_id,
            &subroutine_image_id,
        )
        .await
        .unwrap();
        assert_eq!(subroutines, vec![mock_subroutine_entity]);
    }
}
