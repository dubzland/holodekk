use async_trait::async_trait;

use log::trace;

use crate::core::projectors::ProjectorsServiceMethods;
use crate::core::subroutine_definitions::SubroutineDefinitionsServiceMethods;
use crate::core::subroutines::{
    entities::SubroutineEntity,
    repositories::{SubroutinesQuery, SubroutinesRepository},
    FindSubroutines, Result, SubroutinesFindInput,
};

use super::SubroutinesService;

impl<'a> From<&'a SubroutinesFindInput<'a>> for SubroutinesQuery<'a> {
    fn from(value: &'a SubroutinesFindInput) -> Self {
        let mut query = SubroutinesQuery::builder();
        if let Some(projector_id) = value.projector_id() {
            query.for_projector(projector_id);
        }
        if let Some(subroutine_definition_id) = value.subroutine_definition_id() {
            query.for_subroutine_definition(subroutine_definition_id);
        }
        query.build()
    }
}

#[async_trait]
impl<R, P, D> FindSubroutines for SubroutinesService<R, P, D>
where
    R: SubroutinesRepository,
    P: ProjectorsServiceMethods,
    D: SubroutineDefinitionsServiceMethods,
{
    async fn find<'a>(&self, input: &'a SubroutinesFindInput<'a>) -> Result<Vec<SubroutineEntity>> {
        trace!("SubroutinesService.find({:?})", input);
        let query = SubroutinesQuery::from(input);
        let subroutines = self.repo.subroutines_find(&query).await?;
        Ok(subroutines)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::core::projectors::fixtures::{mock_projectors_service, MockProjectorsService};
    use crate::core::subroutine_definitions::fixtures::{
        mock_subroutine_definitions_service, MockSubroutineDefinitionsService,
    };
    use crate::core::subroutines::{
        entities::{fixtures::subroutine, SubroutineEntity},
        repositories::{fixtures::subroutines_repository, MockSubroutinesRepository},
        Result,
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn executes_query(
        mock_projectors_service: MockProjectorsService,
        mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mut subroutines_repository: MockSubroutinesRepository,
        subroutine: SubroutineEntity,
    ) -> Result<()> {
        let (director_tx, _director_rx) = tokio::sync::mpsc::channel(1);

        let projector_id = subroutine.projector_id().to_owned();
        let definition_id = subroutine.subroutine_definition_id().to_owned();
        let subroutines_find_result = Ok(vec![]);
        subroutines_repository
            .expect_subroutines_find()
            .withf(move |query: &SubroutinesQuery| {
                query.projector_id().unwrap() == &projector_id
                    && query.subroutine_definition_id().unwrap() == &definition_id
            })
            .return_once(move |_| subroutines_find_result);

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            director_tx,
            Arc::new(mock_projectors_service),
            Arc::new(mock_subroutine_definitions_service),
        );

        service
            .find(&SubroutinesFindInput::new(
                Some(subroutine.projector_id()),
                Some(subroutine.subroutine_definition_id()),
            ))
            .await?;
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn returns_results_of_query(
        mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mock_projectors_service: MockProjectorsService,
        mut subroutines_repository: MockSubroutinesRepository,
        subroutine: SubroutineEntity,
    ) -> Result<()> {
        let (director_tx, _director_rx) = tokio::sync::mpsc::channel(1);

        let subroutines_find_result = Ok(vec![subroutine.clone()]);
        subroutines_repository
            .expect_subroutines_find()
            .return_once(move |_| subroutines_find_result);

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            director_tx,
            Arc::new(mock_projectors_service),
            Arc::new(mock_subroutine_definitions_service),
        );

        let subroutines = service
            .find(&SubroutinesFindInput::new(
                Some(subroutine.projector_id()),
                Some(subroutine.subroutine_definition_id()),
            ))
            .await?;
        assert_eq!(subroutines, vec![subroutine]);
        Ok(())
    }
}
