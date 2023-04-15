use crate::core::projectors::repositories::{projector_repo_id, ProjectorsRepository};
use crate::core::services::Result;
use async_trait::async_trait;

use super::{ProjectorExists, ProjectorsExistsInput, ProjectorsService};

#[async_trait]
impl<R> ProjectorExists for ProjectorsService<R>
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
