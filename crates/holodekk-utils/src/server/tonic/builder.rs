pub use futures_core::future::BoxFuture;

use std::{net::Ipv4Addr, path::Path};

use super::{ListenerConfig, TonicServer, TonicService};

pub struct TonicServerBuilder<T>
where
    T: TonicService,
{
    service: T,
}

impl<T> TonicServerBuilder<T>
where
    T: TonicService,
{
    pub fn new(service: T) -> Self {
        Self { service }
    }

    pub fn listen_tcp(self, port: &u16, addr: Option<&Ipv4Addr>) -> TonicServer {
        self.build(ListenerConfig::tcp(port, addr))
    }

    pub fn listen_uds<P: AsRef<Path>>(self, socket: P) -> TonicServer {
        self.build(ListenerConfig::uds(socket))
    }

    pub fn listen<P: AsRef<Path>>(
        self,
        port: Option<&u16>,
        addr: Option<&Ipv4Addr>,
        socket: Option<P>,
    ) -> TonicServer {
        self.build(ListenerConfig::from_options(port, addr, socket).unwrap())
    }

    fn build(self, listener_config: ListenerConfig) -> TonicServer {
        TonicServer::new(listener_config, self.service)
    }
}
