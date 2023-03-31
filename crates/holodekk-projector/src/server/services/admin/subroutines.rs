use tonic::{Request, Response, Status};

use super::{Empty, Subroutine, SubroutineList, Subroutines};

#[derive(Default)]
pub struct SubroutinesService {}

#[tonic::async_trait]
impl Subroutines for SubroutinesService {
    async fn list(&self, _request: Request<Empty>) -> Result<Response<SubroutineList>, Status> {
        let sub = Subroutine {
            name: "acme/widgets".to_string(),
            pid: 123,
        };
        Ok(Response::new(SubroutineList {
            subroutines: vec![sub],
        }))
    }
}
