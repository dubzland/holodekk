use tonic::{Request, Response};

use crate::apis::grpc::subroutines::proto::{
    entities::{RpcStatusRequest, RpcSubroutineStatus},
    RpcSubroutines,
};
use crate::services::subroutines::Status;

use super::SubroutinesApiServer;

#[tonic::async_trait]
impl<S> RpcSubroutines for SubroutinesApiServer<S>
where
    S: Status + Send + 'static,
{
    async fn status(
        &self,
        request: Request<RpcStatusRequest>,
    ) -> std::result::Result<Response<RpcSubroutineStatus>, tonic::Status> {
        // look up the subroutine instance by name
        let status = self.service.status(&request.into_inner().name).await?;
        let response: RpcSubroutineStatus = status.into();

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::*;
    use rstest::*;

    use crate::entities::SubroutineStatus;
    use crate::services::{subroutines::MockStatus, Error};

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn returns_subroutine_status() -> std::result::Result<(), tonic::Status> {
        let mut service = MockStatus::default();
        service
            .expect_status()
            .with(eq("test"))
            .return_const(Ok(SubroutineStatus::Stopped));
        let api_server = SubroutinesApiServer::new(Arc::new(service));
        let request = tonic::Request::new(RpcStatusRequest {
            name: "test".into(),
        });
        let status = api_server.status(request).await?.into_inner();
        assert_eq!(status, RpcSubroutineStatus::from(SubroutineStatus::Stopped));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn returns_error_when_service_fails() -> std::result::Result<(), tonic::Status> {
        let mut service = MockStatus::default();
        service
            .expect_status()
            .with(eq("test"))
            .return_const(Err(Error::NotFound));
        let api_server = SubroutinesApiServer::new(Arc::new(service));
        let request = tonic::Request::new(RpcStatusRequest {
            name: "test".into(),
        });
        let res = api_server.status(request).await;
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
        Ok(())
    }
}
