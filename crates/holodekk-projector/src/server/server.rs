use std::net::SocketAddr;

use tokio::sync::mpsc::unbounded_channel;
use tokio::task::JoinHandle;

use tonic::transport::Server;

use crate::error::Result;

use super::{hello_world::greeter_server::GreeterServer, MyGreeter};

use super::handle::{ProjectorCommand, ProjectorHandle};
use super::runtime::ProjectorRuntime;

pub struct ProjectorServer {
    runtime: ProjectorRuntime,
    _task_handle: Option<JoinHandle<std::result::Result<(), tonic::transport::Error>>>,
}

impl ProjectorServer {
    pub fn new() -> Self {
        Self {
            runtime: ProjectorRuntime::create(),
            _task_handle: None,
        }
    }

    pub(crate) fn runtime(&self) -> &ProjectorRuntime {
        &self.runtime
    }

    pub fn listen_tcp(self, port: u16, addr: Option<&str>) -> Result<ProjectorHandle> {
        let (cmd_tx, cmd_rx) = unbounded_channel();

        let signal = async {
            let mut async_rx = cmd_rx;
            let res = async_rx.recv().await;
            match res {
                Some(cmd) => match cmd {
                    ProjectorCommand::Stop { completion } => {
                        if let Some(tx) = completion {
                            let _ = tx.send(());
                        }
                    }
                },
                None => (),
            };
        };

        let listen_address: SocketAddr;

        if addr.is_some() {
            let addr = addr.unwrap();
            listen_address = format!("{}:{}", addr, port).parse().unwrap();
        } else {
            let addr = "[::1]";
            listen_address = format!("{}:{}", addr, port).parse().unwrap();
        }

        let server = Server::builder()
            .add_service(GreeterServer::new(MyGreeter::default()))
            .serve_with_shutdown(listen_address, signal);

        let server_handle = self.runtime.spawn_server(server);

        Ok(ProjectorHandle::new(
            ProjectorServer {
                _task_handle: Some(server_handle),
                ..self
            },
            cmd_tx,
        ))
    }
}
