mod create;

use std::sync::Arc;

use crate::core::subroutines::CreateSubroutine;

use super::proto::RpcSubroutinesServer;

#[derive(Clone, Debug)]
pub struct SubroutinesApiServer<S>
where
    S: CreateSubroutine + Send + 'static,
{
    service: Arc<S>,
}

impl<S> SubroutinesApiServer<S>
where
    S: CreateSubroutine + Send + 'static,
{
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

pub fn subroutines_api_server<S>(service: Arc<S>) -> RpcSubroutinesServer<SubroutinesApiServer<S>>
where
    S: CreateSubroutine + Send + Sync + 'static,
{
    RpcSubroutinesServer::new(SubroutinesApiServer::new(service))
}
