use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::core::{entities, repositories::ProjectorRepository};

use super::ProjectorsService;

#[derive(Clone, Debug)]
pub struct ProjectorExistsInput {
    pub namespace: String,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Exists {
    /// Checks for the existence of a [Projector](entities::Projector)
    ///
    /// # Arguments
    ///
    /// `input` ([ProjectorExistsInput]) - parameters for the projector (currently only `namespace`)
    async fn exists(&self, input: ProjectorExistsInput) -> bool;
}

#[async_trait]
impl<T> Exists for ProjectorsService<T>
where
    T: ProjectorRepository,
{
    async fn exists(&self, input: ProjectorExistsInput) -> bool {
        let id = entities::projector::generate_id(&self.fleet, &input.namespace);
        self.repo.projector_exists(&id).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::config::fixtures::{mock_config, MockConfig};
    use crate::core::repositories::{fixtures::projector_repository, MockProjectorRepository};

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_true_for_existing_projector(
        mock_config: MockConfig,
        mut projector_repository: MockProjectorRepository,
    ) {
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::channel(1);

        projector_repository
            .expect_projector_exists()
            .return_const(true);

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projector_repository),
            cmd_tx,
        );

        assert!(
            service
                .exists(ProjectorExistsInput {
                    namespace: "existing".to_string()
                })
                .await
        );
    }

    #[rstest]
    #[tokio::test]
    async fn returns_false_for_nonexisting_projector(
        mock_config: MockConfig,
        mut projector_repository: MockProjectorRepository,
    ) {
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::channel(1);

        projector_repository
            .expect_projector_exists()
            .return_const(false);

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projector_repository),
            cmd_tx,
        );

        assert!(
            !service
                .exists(ProjectorExistsInput {
                    namespace: "existing".to_string()
                })
                .await
        );
    }
}
