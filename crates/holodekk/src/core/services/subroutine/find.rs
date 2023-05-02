use async_trait::async_trait;
use log::trace;

use crate::core::images::SubroutineImageId;
use crate::entities::{
    SceneEntityId, SubroutineEntity, SubroutineEntityRepository, SubroutineEntityRepositoryQuery,
};

use super::{EntityServiceResult, FindSubroutines, FindSubroutinesInput, SubroutineEntityService};

#[async_trait]
impl<R> FindSubroutines for SubroutineEntityService<R>
where
    R: SubroutineEntityRepository,
{
    async fn find<'a>(
        &self,
        input: &'a FindSubroutinesInput<'a>,
    ) -> EntityServiceResult<Vec<SubroutineEntity>> {
        trace!("SubroutineEntityService::find({:?})", input);

        let mut query = SubroutineEntityRepositoryQuery::builder();
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

    use crate::core::images::ImageName;
    use crate::entities::{
        fixtures::{mock_subroutine_entity, mock_subroutine_entity_repository},
        MockSubroutineEntityRepository, SubroutineEntity,
    };
    use rstest::*;

    use super::*;

    async fn execute(
        repo: MockSubroutineEntityRepository,
        scene: &str,
        image: &str,
    ) -> EntityServiceResult<Vec<SubroutineEntity>> {
        let service = SubroutineEntityService::new(Arc::new(repo));

        service
            .find(&FindSubroutinesInput::new(Some(scene), Some(image)))
            .await
    }

    #[rstest]
    #[tokio::test]
    async fn executes_query(mut mock_subroutine_entity_repository: MockSubroutineEntityRepository) {
        let scene_entity_id = SceneEntityId::generate();
        let subroutine_image_id = SubroutineImageId::generate(&ImageName::from("test"));

        {
            let scene_entity_id = scene_entity_id.clone();
            let subroutine_image_id = subroutine_image_id.clone();
            mock_subroutine_entity_repository
                .expect_subroutines_find()
                .withf(move |query: &SubroutineEntityRepositoryQuery| {
                    query.scene_entity_id == Some(&scene_entity_id)
                        && query.subroutine_image_id == Some(&subroutine_image_id)
                })
                .return_once(|_| Ok(vec![]));
        }

        execute(
            mock_subroutine_entity_repository,
            &scene_entity_id,
            &subroutine_image_id,
        )
        .await
        .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_results_of_query(
        mut mock_subroutine_entity_repository: MockSubroutineEntityRepository,
        mock_subroutine_entity: SubroutineEntity,
    ) {
        let scene_entity_id = mock_subroutine_entity.scene_entity_id.clone();
        let subroutine_image_id = mock_subroutine_entity.subroutine_image_id.clone();

        {
            let mock_subroutine_entity = mock_subroutine_entity.clone();
            mock_subroutine_entity_repository
                .expect_subroutines_find()
                .return_once(move |_| Ok(vec![mock_subroutine_entity]));
        }

        let subroutines = execute(
            mock_subroutine_entity_repository,
            &scene_entity_id,
            &subroutine_image_id,
        )
        .await
        .unwrap();
        assert_eq!(subroutines, vec![mock_subroutine_entity]);
    }
}
