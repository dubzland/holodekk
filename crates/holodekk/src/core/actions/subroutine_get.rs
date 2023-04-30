use std::sync::Arc;

use crate::core::{
    entities::{SubroutineEntity, SubroutineEntityId},
    repositories::{self, SubroutinesRepository},
};

#[derive(Clone, Debug)]
pub struct Request<'a> {
    pub id: &'a SubroutineEntityId,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Subroutine not found with id {0}")]
    NotFound(SubroutineEntityId),
    #[error("General repository error occurred")]
    Repository(#[from] repositories::Error),
}

pub type Result = std::result::Result<SubroutineEntity, Error>;

pub async fn execute<R>(repo: Arc<R>, request: Request<'_>) -> Result
where
    R: SubroutinesRepository,
{
    let subroutine = repo.subroutines_get(request.id).await.map_err(|err| {
        if matches!(err, repositories::Error::NotFound(..)) {
            Error::NotFound(request.id.to_owned())
        } else {
            Error::from(err)
        }
    })?;
    Ok(subroutine)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::eq;
    use rstest::*;

    use crate::core::entities::fixtures::mock_subroutine;
    use crate::core::repositories::{
        fixtures::mock_subroutines_repository, MockSubroutinesRepository,
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_nonexisting_subroutine(
        mut mock_subroutines_repository: MockSubroutinesRepository,
    ) {
        let id = SubroutineEntityId::generate();

        {
            let sub_id = id.clone();
            mock_subroutines_repository
                .expect_subroutines_get()
                .with(eq(sub_id))
                .return_once(|id| Err(repositories::Error::NotFound(id.to_owned())));
        }

        assert!(matches!(
            execute(Arc::new(mock_subroutines_repository), Request { id: &id })
                .await
                .unwrap_err(),
            Error::NotFound(..)
        ));
    }

    #[rstest]
    #[tokio::test]
    async fn returns_subroutine_for_existing_subroutine(
        mut mock_subroutines_repository: MockSubroutinesRepository,
        mock_subroutine: SubroutineEntity,
    ) {
        let id = mock_subroutine.id.clone();

        {
            let sub = mock_subroutine.clone();
            mock_subroutines_repository
                .expect_subroutines_get()
                .return_once(move |_| Ok(sub.clone()));
        }

        assert_eq!(
            execute(Arc::new(mock_subroutines_repository), Request { id: &id })
                .await
                .unwrap(),
            mock_subroutine
        );
    }
}
