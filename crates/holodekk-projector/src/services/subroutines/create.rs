use std::path::PathBuf;

use crate::entities::Subroutine;
use crate::repository::Repository;
use crate::services::{Error, Result};

use super::SubroutinesService;

#[derive(Clone, Debug)]
pub struct CreateSubroutineInput {
    pub name: String,
    pub path: PathBuf,
}

impl<T> SubroutinesService<T>
where
    T: Repository,
{
    /// Creates a Subroutine entry in the repository.
    pub async fn create(&self, input: CreateSubroutineInput) -> Result<Subroutine> {
        // make sure this subroutine does not already exist
        println!("Checking for subroutine with name: {}", input.name);
        if self.repo.get_subroutine_by_name(&input.name).await.is_ok() {
            return Err(Error::Duplicate);
        }

        let subroutine = Subroutine {
            name: input.name.to_owned(),
            path: input.path.to_owned(),
        };
        let subroutine = self.repo.create_subroutine(&subroutine).await?;
        Ok(subroutine)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::*;
    use rstest::*;

    use super::*;

    use crate::repository::MockRepository;
    use crate::services::Error;

    #[fixture]
    fn repository() -> MockRepository {
        MockRepository::new()
    }

    #[fixture]
    fn subroutine() -> Subroutine {
        Subroutine {
            name: "test".to_string(),
            path: PathBuf::from("/tmp"),
        }
    }

    #[rstest]
    #[tokio::test]
    async fn creates_subroutine(mut repository: MockRepository, subroutine: Subroutine) {
        let input = CreateSubroutineInput {
            name: "test".into(),
            path: "/tmp".into(),
        };

        repository
            .expect_get_subroutine_by_name()
            .with(eq("test"))
            .returning(|_| Err(crate::repository::Error::NotFound));

        let sub_result = subroutine.clone();

        repository
            .expect_create_subroutine()
            .withf(|new_sub: &Subroutine| {
                (*new_sub).name.eq("test") && (*new_sub).path.eq(&PathBuf::from("/tmp"))
            })
            .returning(move |_| Ok(subroutine.clone()));

        let service = SubroutinesService::new(Arc::new(repository));

        let res = service.create(input).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), sub_result);
    }

    #[rstest]
    #[tokio::test]
    async fn rejects_duplicate_subroutine_name(
        mut repository: MockRepository,
        subroutine: Subroutine,
    ) {
        let input = CreateSubroutineInput {
            name: "test".into(),
            path: "/tmp".into(),
        };

        repository
            .expect_get_subroutine_by_name()
            .with(eq("test"))
            .returning(move |_| Ok(subroutine.clone()));

        let service = SubroutinesService::new(Arc::new(repository));

        let res = service.create(input).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::Duplicate);
    }
}
