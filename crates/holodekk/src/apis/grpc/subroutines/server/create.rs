use tonic::{Request, Response};

use crate::core::services::subroutines::{Create, SubroutinesCreateInput};

use crate::apis::grpc::subroutines::proto::{
    entities::{RpcCreateRequest, RpcSubroutine, RpcSubroutineKind},
    RpcSubroutines,
};

use super::SubroutinesApiServer;

#[tonic::async_trait]
impl<S> RpcSubroutines for SubroutinesApiServer<S>
where
    S: Create + Send + Sync + 'static,
{
    async fn create(
        &self,
        request: Request<RpcCreateRequest>,
    ) -> std::result::Result<Response<RpcSubroutine>, tonic::Status> {
        let request = request.into_inner();
        let input = SubroutinesCreateInput {
            name: request.name,
            path: request.path.into(),
            kind: RpcSubroutineKind::from_i32(request.kind).unwrap().into(),
        };
        let instance = self.service.create(input).await?;

        Ok(Response::new(instance.into()))
    }
}

// #[cfg(test)]
// mod tests {
//     use std::sync::Arc;

//     use mockall::predicate::*;
//     use rstest::*;

//     use crate::core::entities::SubroutineStatus;
//     use crate::core::services::{subroutines::MockStatus, Error};

//     use super::*;

//     #[rstest]
//     #[tokio::test]
//     async fn returns_subroutine_status() -> std::result::Result<(), tonic::Status> {
//         let mut service = MockStatus::default();
//         service
//             .expect_status()
//             .with(eq("test"))
//             .return_const(Ok(SubroutineStatus::Stopped));
//         let api_server = SubroutinesApiServer::new(Arc::new(service));
//         let request = tonic::Request::new(RpcStatusRequest {
//             name: "test".into(),
//         });
//         let status = api_server.status(request).await?.into_inner();
//         assert_eq!(status, RpcSubroutineStatus::from(SubroutineStatus::Stopped));
//         Ok(())
//     }

//     #[rstest]
//     #[tokio::test]
//     async fn returns_error_when_service_fails() -> std::result::Result<(), tonic::Status> {
//         let mut service = MockStatus::default();
//         service
//             .expect_status()
//             .with(eq("test"))
//             .return_const(Err(Error::NotFound));
//         let api_server = SubroutinesApiServer::new(Arc::new(service));
//         let request = tonic::Request::new(RpcStatusRequest {
//             name: "test".into(),
//         });
//         let res = api_server.status(request).await;
//         assert!(res.is_err());
//         let err = res.unwrap_err();
//         assert_eq!(err.code(), tonic::Code::NotFound);
//         Ok(())
//     }
// }
