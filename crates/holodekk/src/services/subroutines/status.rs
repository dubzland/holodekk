use crate::entities::SubroutineStatus;
use crate::repositories::Repository;
use crate::services::{Error, Result};

use super::SubroutinesService;

impl<T> SubroutinesService<T>
where
    T: Repository,
{
    pub async fn status(&self, name: &str) -> Result<SubroutineStatus> {
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
            if let Some(instance) = instances.iter().find(|i| {
                println!("i.fleet:      {}", i.fleet);
                println!("config.fleet: {}", self.config.fleet);
                i.fleet == self.config.fleet && i.namespace == self.namespace
            }) {
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

    // use crate::entities::fixtures::{subroutine, subroutine_instance, subroutine_with_instance};
    // use crate::entities::{Subroutine, SubroutineInstance, SubroutineStatus};
    // use crate::repositories::{fixtures::repository, MockRepository};
    // use crate::services::Error;
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
}
