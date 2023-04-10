mod status;

use std::sync::Arc;

use crate::services::subroutines::Status;

use super::proto::RpcSubroutinesServer;

#[derive(Clone, Debug)]
pub struct SubroutinesApiServer<S>
where
    S: Status + Send + 'static,
{
    service: Arc<S>,
}

impl<S> SubroutinesApiServer<S>
where
    S: Status + Send + 'static,
{
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

pub fn subroutines_api_server<S>(service: Arc<S>) -> RpcSubroutinesServer<SubroutinesApiServer<S>>
where
    S: Status + Send + 'static,
{
    RpcSubroutinesServer::new(SubroutinesApiServer::new(service))
}
