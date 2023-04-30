use std::sync::Arc;

use log::trace;

use crate::core::{
    entities::SubroutineEntityId,
    repositories::{self, SubroutinesRepository},
};

#[derive(Clone, Debug)]
pub struct Request<'a> {
    pub id: &'a SubroutineEntityId,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Subroutine not found")]
    NotFound(SubroutineEntityId),
    #[error("General repository error occurred")]
    Repository(#[from] repositories::Error),
}

pub type Result = std::result::Result<(), Error>;

pub async fn execute<R>(repo: Arc<R>, request: Request<'_>) -> Result
where
    R: SubroutinesRepository,
{
    trace!("delete_subroutine::execute({:?})", request);

    // ensure the subroutine exists
    let subroutine = repo
        .subroutines_get(request.id)
        .await
        .map_err(|err| match err {
            repositories::Error::NotFound(id) => Error::NotFound(id),
            _ => Error::from(err),
        })?;

    // remove subroutine from the repository
    repo.subroutines_delete(&subroutine.id).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::eq;
    use rstest::*;

    use crate::core::{
        entities::{fixtures::mock_subroutine_entity, SubroutineEntity},
        repositories::{fixtures::mock_subroutines_repository, MockSubroutinesRepository},
    };

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_error_for_non_existent_subroutine(
        mut mock_subroutines_repository: MockSubroutinesRepository,
    ) {
        let mock_id = SubroutineEntityId::generate();

        // subroutine does not exist
        mock_subroutines_repository
            .expect_subroutines_get()
            .with(eq(mock_id.clone()))
            .return_once(|id| Err(repositories::Error::NotFound(id.to_owned())));

        let res = execute(
            Arc::new(mock_subroutines_repository),
            Request { id: &mock_id },
        )
        .await;

        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::NotFound(..)));
    }

    #[rstest]
    #[tokio::test]
    async fn removes_entry_in_repository(
        mut mock_subroutines_repository: MockSubroutinesRepository,
        mock_subroutine_entity: SubroutineEntity,
    ) {
        {
            let sub = mock_subroutine_entity.clone();
            let sub_id = sub.id.clone();
            mock_subroutines_repository
                .expect_subroutines_get()
                .withf(move |id| id == &sub_id)
                .return_once(move |_| Ok(sub));
        }

        {
            let sub_id = mock_subroutine_entity.id.clone();
            mock_subroutines_repository
                .expect_subroutines_delete()
                .withf(move |id| id == &sub_id)
                .return_once(|_| Ok(()));
        }

        execute(
            Arc::new(mock_subroutines_repository),
            Request {
                id: &mock_subroutine_entity.id,
            },
        )
        .await
        .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn returns_ok(
        mut mock_subroutines_repository: MockSubroutinesRepository,
        mock_subroutine_entity: SubroutineEntity,
    ) {
        {
            let sub = mock_subroutine_entity.clone();
            mock_subroutines_repository
                .expect_subroutines_get()
                .return_once(move |_| Ok(sub));
        }

        mock_subroutines_repository
            .expect_subroutines_delete()
            .return_once(|_| Ok(()));

        let res = execute(
            Arc::new(mock_subroutines_repository),
            Request {
                id: &mock_subroutine_entity.id,
            },
        )
        .await;

        assert!(res.is_ok());
    }
}
