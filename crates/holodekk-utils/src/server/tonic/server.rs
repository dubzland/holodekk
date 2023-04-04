pub use futures_core::future::BoxFuture;

use tokio::sync::oneshot::channel;

use super::{ListenerConfig, Server, TonicServerHandle, TonicService};

pub struct TonicServer {
    listener_config: ListenerConfig,
    service: Box<dyn TonicService>,
}

impl TonicServer {
    pub(crate) fn new(listener_config: ListenerConfig, service: impl TonicService) -> Self {
        Self {
            listener_config,
            service: Box::new(service),
        }
    }
}

impl Server for TonicServer {
    type Handle = TonicServerHandle;

    fn listen(&self) -> TonicServerHandle {
        let (cmd_tx, cmd_rx) = channel::<()>();
        let fut = self.service.listen(&self.listener_config, cmd_rx);
        TonicServerHandle::new(cmd_tx, fut)
    }
}
