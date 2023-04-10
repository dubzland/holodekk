use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::entities::SubroutineStatus;
use crate::repositories::Repository;
use crate::services::{Error, Result};

use super::SubroutinesService;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Status: Sync {
    async fn status(&self, name: &str) -> Result<SubroutineStatus>;
}

#[async_trait]
impl<T> Status for SubroutinesService<T>
where
    T: Repository,
{
    /// Retrieves the status for a subroutine instance.
    ///
    /// Scopes to the fleet/namespace of the service.
    ///
    /// # Arguments
    ///
    /// `name` - Name of the subroutine
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use std::sync::Arc;
    /// # use holodekk::entities::SubroutineKind;
    /// # use holodekk::services::subroutines::{Create, SubroutineCreateInput};
    /// use holodekk::HolodekkConfig;
    /// use holodekk::services::subroutines::{Status, SubroutinesService};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let config = HolodekkConfig {
    ///     fleet: "test".into(),
    ///     root_path: "/tmp".into(),
    ///     bin_path: "/tmp".into(),
    /// };
    /// let repo = holodekk::repositories::memory::MemoryRepository::default();
    /// let service = SubroutinesService::new(Arc::new(config), Arc::new(repo), "test");
    /// # service.create(SubroutineCreateInput {
    /// #     name: "acme/widget-app".into(),
    /// #     path: "/tmp".into(),
    /// #     kind: SubroutineKind::Ruby,
    /// #  }).await.unwrap();
    /// let status = service.status("acme/widget-app").await.unwrap();
    /// # }
    /// ```
    async fn status(&self, name: &str) -> Result<SubroutineStatus> {
        let subroutine =
            self.repo
                .subroutine_get_by_name(name, true)
                .await
                .map_err(|e| match e {
                    crate::repositories::Error::NotFound => Error::NotFound,
                    _ => Error::Repository(e),
                })?;

        if let Some(instances) = subroutine.instances {
            println!("instances: {:#?}", instances);
            // Scan the instances for one matching our fleet/namespace
            if let Some(instance) = instances
                .iter()
                .find(|i| i.fleet == self.config.fleet && i.namespace == self.namespace)
            {
                Ok(instance.status)
            } else {
                Err(Error::NotFound)
            }
        } else {
            Err(Error::NotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rstest::*;

    use crate::{
        entities::{
            subroutine::fixtures::{subroutine, subroutine_with_instance},
            subroutine::instance::fixtures::subroutine_instance,
            Subroutine, SubroutineInstance, SubroutineStatus,
        },
        fixtures::holodekk_config,
        repositories::{fixtures::repository, MockRepository},
        services::Error,
        HolodekkConfig,
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_status_for_existing_subroutine_instance(
        holodekk_config: HolodekkConfig,
        mut repository: MockRepository,
        subroutine_with_instance: Subroutine,
        subroutine_instance: SubroutineInstance,
    ) -> Result<()> {
        let namespace = subroutine_instance.namespace.clone();
        let name = subroutine_with_instance.name.clone();

        repository
            .expect_subroutine_get_by_name()
            .withf(move |sub_name, include| sub_name == &name && include == &true)
            .return_const(Ok(subroutine_with_instance.clone()));

        let service = SubroutinesService::new(
            Arc::new(holodekk_config),
            Arc::new(repository),
            namespace.clone(),
        );

        let status = service.status(&subroutine_with_instance.name).await?;
        assert_eq!(status, SubroutineStatus::Unknown);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn returns_not_found_for_missing_subroutine(
        holodekk_config: HolodekkConfig,
        mut repository: MockRepository,
        subroutine: Subroutine,
        subroutine_instance: SubroutineInstance,
    ) -> Result<()> {
        let namespace = subroutine_instance.namespace.clone();
        let name = subroutine.name.clone();

        repository
            .expect_subroutine_get_by_name()
            .withf(move |sub_name, include| sub_name == name && include == &true)
            .return_const(Err(crate::repositories::Error::NotFound));

        let service = SubroutinesService::new(
            Arc::new(holodekk_config),
            Arc::new(repository),
            namespace.clone(),
        );

        let res = service.status(&subroutine.name).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::NotFound);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn returns_not_found_when_no_instances_running(
        holodekk_config: HolodekkConfig,
        mut repository: MockRepository,
        subroutine: Subroutine,
    ) -> Result<()> {
        let name = subroutine.name.clone();

        repository
            .expect_subroutine_get_by_name()
            .withf(move |sub_name, include| sub_name == &name && include == &true)
            .return_const(Ok(subroutine.clone()));

        let service =
            SubroutinesService::new(Arc::new(holodekk_config), Arc::new(repository), "test");

        let res = service.status(&subroutine.name).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::NotFound);
        Ok(())
    }
}
