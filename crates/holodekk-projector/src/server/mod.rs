mod handle;
pub use handle::{ProjectorCommand, ProjectorHandle};

mod runtime;
use std::net::{SocketAddr, TcpListener};

use hello_world::{greeter_server::Greeter, HelloReply, HelloRequest};

use hello_world::greeter_server::GreeterServer;

use tokio::sync::mpsc::unbounded_channel;
use tokio::task::JoinHandle;

use tonic::transport::Server;
use tonic::{Request, Response, Status};

use runtime::ProjectorRuntime;

use crate::error::Result;

pub(crate) mod hello_world {
    tonic::include_proto!("helloworld");
}

#[derive(Default)]
pub(crate) struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> std::result::Result<Response<HelloReply>, Status> {
        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

pub struct ProjectorServer {
    runtime: ProjectorRuntime,
    _task_handle: Option<JoinHandle<std::result::Result<(), tonic::transport::Error>>>,
}

impl Default for ProjectorServer {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn listen_tcp(self, port: Option<u16>, addr: Option<&str>) -> Result<ProjectorHandle> {
        let (cmd_tx, cmd_rx) = unbounded_channel();

        let signal = async {
            let mut async_rx = cmd_rx;
            let res = async_rx.recv().await;
            if let Some(cmd) = res {
                match cmd {
                    ProjectorCommand::Stop { completion } => {
                        if let Some(tx) = completion {
                            let _ = tx.send(());
                        }
                    }
                }
            }
        };

        let addr = addr.unwrap_or("[::1]");
        let port = port
            .or_else(|| {
                let listener = TcpListener::bind(format!("{addr}:0")).unwrap();
                Some(listener.local_addr().unwrap().port())
            })
            .unwrap();
        let listen_address: SocketAddr = format!("{}:{}", addr, port).parse().unwrap();

        let server = Server::builder()
            .add_service(GreeterServer::new(MyGreeter::default()))
            .serve_with_shutdown(listen_address, signal);

        let server_handle = self.runtime.spawn_server(server);

        Ok(ProjectorHandle::new(
            ProjectorServer {
                _task_handle: Some(server_handle),
                ..self
            },
            port,
            cmd_tx,
        ))
    }
}
