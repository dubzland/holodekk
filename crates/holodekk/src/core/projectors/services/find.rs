use async_trait::async_trait;
use log::trace;

use crate::core::projectors::entities::Projector;
use crate::core::projectors::repositories::{ProjectorsQuery, ProjectorsRepository};
use crate::core::services::Result;

use super::{FindProjectors, ProjectorsFindInput, ProjectorsService};

impl From<ProjectorsFindInput> for ProjectorsQuery {
    fn from(value: ProjectorsFindInput) -> Self {
        let mut query = ProjectorsQuery::builder();
        if let Some(fleet) = value.fleet {
            query.fleet_eq(&fleet);
        }
        query.build()
    }
}

#[async_trait]
impl<R> FindProjectors for ProjectorsService<R>
where
    R: ProjectorsRepository,
{
    async fn find(&self, input: ProjectorsFindInput) -> Result<Vec<Projector>> {
        trace!("ProjectorsService.find()");
        let projectors = self.repo.projectors_find(input.into()).await?;
        Ok(projectors)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::config::fixtures::{mock_config, MockConfig};
    use crate::core::projectors::{
        entities::{fixtures::projector, Projector},
        repositories::{fixtures::projectors_repository, MockProjectorsRepository},
    };
    use crate::core::services::Result;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn executes_query(
        mock_config: MockConfig,
        mut projectors_repository: MockProjectorsRepository,
    ) -> Result<()> {
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::channel(1);

        projectors_repository
            .expect_projectors_find()
            .withf(|query: &ProjectorsQuery| query.fleet == None)
            .return_const(Ok(vec![]));

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projectors_repository),
            cmd_tx,
        );

        service.find(ProjectorsFindInput::default()).await?;
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn returns_results_of_query(
        mock_config: MockConfig,
        mut projectors_repository: MockProjectorsRepository,
        projector: Projector,
    ) -> Result<()> {
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::channel(1);

        projectors_repository
            .expect_projectors_find()
            .return_const(Ok(vec![projector.clone()]));

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projectors_repository),
            cmd_tx,
        );

        let projectors = service.find(ProjectorsFindInput::default()).await?;
        assert_eq!(projectors, vec![projector]);
        Ok(())
    }
}
