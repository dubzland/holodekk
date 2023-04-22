use async_trait::async_trait;
use log::trace;

use crate::core::projectors::{
    entities::ProjectorEntity,
    repositories::{ProjectorsQuery, ProjectorsRepository},
    FindProjectors, ProjectorsFindInput, Result,
};

use super::ProjectorsService;

impl From<&'_ ProjectorsFindInput<'_>> for ProjectorsQuery {
    fn from(value: &ProjectorsFindInput) -> Self {
        let mut query = ProjectorsQuery::builder();
        if let Some(namespace) = value.namespace() {
            query.namespace_eq(namespace);
        }
        query.build()
    }
}

#[async_trait]
impl<R> FindProjectors for ProjectorsService<R>
where
    R: ProjectorsRepository,
{
    async fn find<'a>(&self, input: &'a ProjectorsFindInput<'a>) -> Result<Vec<ProjectorEntity>> {
        trace!("ProjectorsService.find()");
        let query = ProjectorsQuery::from(input);
        let projectors = self.repo.projectors_find(query).await?;
        Ok(projectors)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::core::projectors::{
        entities::{fixtures::projector, ProjectorEntity},
        repositories::{fixtures::projectors_repository, MockProjectorsRepository},
        Result,
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn executes_query(mut projectors_repository: MockProjectorsRepository) -> Result<()> {
        let (director_tx, _director_rx) = tokio::sync::mpsc::channel(1);
        projectors_repository
            .expect_projectors_find()
            .withf(|query: &ProjectorsQuery| query.namespace().is_none())
            .return_const(Ok(vec![]));

        let service = ProjectorsService::new(Arc::new(projectors_repository), director_tx);

        service.find(&ProjectorsFindInput::default()).await?;
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn returns_results_of_query(
        mut projectors_repository: MockProjectorsRepository,
        projector: ProjectorEntity,
    ) -> Result<()> {
        let (director_tx, _director_rx) = tokio::sync::mpsc::channel(1);
        projectors_repository
            .expect_projectors_find()
            .return_const(Ok(vec![projector.clone()]));

        let service = ProjectorsService::new(Arc::new(projectors_repository), director_tx);

        let projectors = service.find(&ProjectorsFindInput::default()).await?;
        assert_eq!(projectors, vec![projector]);
        Ok(())
    }
}
