use async_trait::async_trait;

use log::trace;

use crate::core::subroutine_definitions::SubroutineDefinitionsServiceMethods;
use crate::core::subroutines::{
    entities::Subroutine,
    repositories::{SubroutinesQuery, SubroutinesRepository},
    worker::SubroutineCommand,
    FindSubroutines, Result, SubroutinesFindInput,
};
use crate::utils::Worker;

use super::SubroutinesService;

impl From<&'_ SubroutinesFindInput<'_>> for SubroutinesQuery {
    fn from(value: &SubroutinesFindInput) -> Self {
        let mut query = SubroutinesQuery::builder();
        if let Some(fleet) = value.fleet() {
            query.fleet_eq(fleet);
        }
        if let Some(namespace) = value.namespace() {
            query.namespace_eq(namespace);
        }
        if let Some(subroutine_definition_id) = value.subroutine_definition_id() {
            query.for_subroutine_definition(subroutine_definition_id);
        }
        query.build()
    }
}

#[async_trait]
impl<R, W, D> FindSubroutines for SubroutinesService<R, W, D>
where
    R: SubroutinesRepository,
    W: Worker<Command = SubroutineCommand>,
    D: SubroutineDefinitionsServiceMethods,
{
    async fn find<'a>(&self, input: &'a SubroutinesFindInput<'a>) -> Result<Vec<Subroutine>> {
        trace!("SubroutinesService.find({:?})", input);
        let query = SubroutinesQuery::from(input);
        Ok(self.repo.subroutines_find(query).await)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::core::subroutine_definitions::fixtures::{
        mock_subroutine_definitions_service, MockSubroutineDefinitionsService,
    };
    use crate::core::subroutines::{
        entities::{fixtures::subroutine, Subroutine},
        repositories::{fixtures::subroutines_repository, MockSubroutinesRepository},
        worker::{fixtures::mock_subroutines_worker, MockSubroutinesWorker},
        Result,
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn executes_query(
        mock_subroutine_definitions_service: MockSubroutineDefinitionsService,
        mut subroutines_repository: MockSubroutinesRepository,
        mock_subroutines_worker: MockSubroutinesWorker,
        subroutine: Subroutine,
    ) -> Result<()> {
        let fleet = Some(subroutine.fleet().to_owned());

        subroutines_repository
            .expect_subroutines_find()
            .withf(move |query: &SubroutinesQuery| query.fleet == fleet)
            .return_const(vec![]);

        let service = SubroutinesService::new(
            Arc::new(subroutines_repository),
            Arc::new(mock_subroutine_definitions_service),
            mock_subroutines_worker,
        );

        service
            .find(&SubroutinesFindInput::new(
                Some(subroutine.fleet()),
                Some(subroutine.namespace()),
                Some(subroutine.subroutine_definition_id()),
            ))
            .await?;
        Ok(())
    }

    //     #[rstest]
    //     #[tokio::test]
    //     async fn returns_results_of_query(
    //         mock_config: MockConfig,
    //         mut projectors_repository: MockProjectorsRepository,
    //         projector: Projector,
    //         mock_projectors_worker: MockProjectorsWorker,
    //     ) -> Result<()> {
    //         projectors_repository
    //             .expect_projectors_find()
    //             .return_const(Ok(vec![projector.clone()]));

    //         let service = ProjectorsService::new(
    //             Arc::new(mock_config),
    //             Arc::new(projectors_repository),
    //             mock_projectors_worker,
    //         );

    //         let projectors = service.find(&ProjectorsFindInput::default()).await?;
    //         assert_eq!(projectors, vec![projector]);
    //         Ok(())
    //     }
}
