use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::core::projectors::repositories::{projector_repo_id, ProjectorsRepository};
use crate::core::services::Result;

use super::ProjectorsService;

#[derive(Clone, Debug)]
pub struct ProjectorsExistsInput {
    pub namespace: String,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Exists {
    /// Checks for the existence of a [Projector](entities::Projector)
    ///
    /// # Arguments
    ///
    /// `input` ([ProjectorsExistsInput]) - parameters for the projector (currently only `namespace`)
    async fn exists(&self, input: ProjectorsExistsInput) -> Result<bool>;
}

#[async_trait]
impl<R> Exists for ProjectorsService<R>
where
    R: ProjectorsRepository,
{
    async fn exists(&self, input: ProjectorsExistsInput) -> Result<bool> {
        let id = projector_repo_id(&self.fleet, &input.namespace);
        let exists = self.repo.projectors_exists(&id).await?;
        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::config::fixtures::{mock_config, MockConfig};
    use crate::core::projectors::repositories::{
        fixtures::projectors_repository, MockProjectorsRepository,
    };
    use crate::core::services::Result;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_true_for_existing_projector(
        mock_config: MockConfig,
        mut projectors_repository: MockProjectorsRepository,
    ) -> Result<()> {
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::channel(1);

        projectors_repository
            .expect_projectors_exists()
            .return_const(Ok(true));

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projectors_repository),
            cmd_tx,
        );

        assert!(
            service
                .exists(ProjectorsExistsInput {
                    namespace: "existing".to_string()
                })
                .await?
        );
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn returns_false_for_nonexisting_projector(
        mock_config: MockConfig,
        mut projectors_repository: MockProjectorsRepository,
    ) -> Result<()> {
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::channel(1);

        projectors_repository
            .expect_projectors_exists()
            .return_const(Ok(false));

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projectors_repository),
            cmd_tx,
        );

        assert!(
            !service
                .exists(ProjectorsExistsInput {
                    namespace: "existing".to_string()
                })
                .await?
        );
        Ok(())
    }
}
