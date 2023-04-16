use async_trait::async_trait;
#[cfg(test)]
use mockall::*;

use crate::core::projectors::{entities::Projector, repositories::ProjectorsRepository};
use crate::core::services::{Error, Result};

use super::ProjectorsService;

#[derive(Clone, Debug)]
pub struct ProjectorsGetInput {
    id: String,
}

impl ProjectorsGetInput {
    pub fn new(id: &str) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait GetProjector {
    async fn get<'a>(&self, input: &'a ProjectorsGetInput) -> Result<Projector>;
}

#[async_trait]
impl<R> GetProjector for ProjectorsService<R>
where
    R: ProjectorsRepository,
{
    async fn get<'a>(&self, input: &'a ProjectorsGetInput) -> Result<Projector> {
        let projector = self.repo.projectors_get(input.id()).await.map_err(|err| {
            if matches!(err, crate::core::repositories::Error::NotFound) {
                Error::NotFound
            } else {
                Error::from(err)
            }
        })?;
        Ok(projector)
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
    use crate::core::services::{Error, Result};

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_projector_for_existing_projector(
        mock_config: MockConfig,
        mut projectors_repository: MockProjectorsRepository,
        projector: Projector,
    ) -> Result<()> {
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::channel(1);
        let id = projector.id().to_string();

        projectors_repository
            .expect_projectors_get()
            .withf(move |i| i == id)
            .return_const(Ok(projector.clone()));

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projectors_repository),
            cmd_tx,
        );

        assert_eq!(
            service
                .get(&ProjectorsGetInput {
                    id: projector.id().to_string()
                })
                .await?,
            projector
        );
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_nonexisting_projector(
        mock_config: MockConfig,
        mut projectors_repository: MockProjectorsRepository,
    ) -> Result<()> {
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::channel(1);

        projectors_repository
            .expect_projectors_get()
            .return_const(Err(crate::core::repositories::Error::NotFound));

        let service = ProjectorsService::new(
            Arc::new(mock_config),
            Arc::new(projectors_repository),
            cmd_tx,
        );

        assert!(matches!(
            service
                .get(&ProjectorsGetInput {
                    id: "nonexistent".to_string(),
                })
                .await
                .unwrap_err(),
            Error::NotFound
        ));
        Ok(())
    }
}
