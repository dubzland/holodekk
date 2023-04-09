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
            // Scan the instances for one matching our fleet/namespace
            if let Some(instance) = instances
                .iter()
                .find(|i| i.fleet == self.fleet && i.namespace == self.namespace)
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

    use crate::entities::fixtures::{subroutine, subroutine_instance, subroutine_with_instance};
    use crate::entities::{Subroutine, SubroutineInstance, SubroutineStatus};
    use crate::repositories::{fixtures::repository, MockRepository};
    use crate::services::Error;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_status_for_existing_subroutine_instance(
        mut repository: MockRepository,
        subroutine_with_instance: &Subroutine,
        subroutine_instance: &SubroutineInstance,
    ) -> Result<()> {
        let fleet = subroutine_instance.fleet.clone();
        let namespace = subroutine_instance.namespace.clone();
        let name = subroutine_with_instance.name.clone();

        repository
            .expect_subroutine_get_by_name()
            .withf(move |sub_name, include| {
                println!("sub_name: {}", sub_name);
                println!("equal?: {}", sub_name == &name);
                println!("include:  {}", include);
                println!("equal?: {}", include == &true);
                sub_name == &name && include == &true
            })
            .return_const(Ok(subroutine_with_instance.clone()));

        let service =
            SubroutinesService::new(Arc::new(repository), fleet.clone(), namespace.clone());

        let status = service.status(&subroutine_with_instance.name).await?;
        assert_eq!(status, SubroutineStatus::Unknown);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn returns_not_found_for_missing_subroutine(
        mut repository: MockRepository,
        subroutine: &Subroutine,
        subroutine_instance: &SubroutineInstance,
    ) -> Result<()> {
        let fleet = subroutine_instance.fleet.clone();
        let namespace = subroutine_instance.namespace.clone();
        let name = subroutine.name.clone();

        repository
            .expect_subroutine_get_by_name()
            .withf(move |sub_name, include| sub_name == name && include == &true)
            .return_const(Err(crate::repositories::Error::NotFound));

        let service =
            SubroutinesService::new(Arc::new(repository), fleet.clone(), namespace.clone());

        let res = service.status(&subroutine.name).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::NotFound);
        Ok(())
    }
}
