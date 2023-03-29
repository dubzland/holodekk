use hello_world::{greeter_server::Greeter, HelloReply, HelloRequest};

pub use hello_world::greeter_server::GreeterServer;

use tonic::{Request, Response, Status};

pub(crate) mod hello_world {
    tonic::include_proto!("helloworld");
}

#[derive(Default)]
pub struct MyGreeter {}

impl MyGreeter {
    pub fn build() -> tonic::transport::server::Router {
        tonic::transport::Server::builder().add_service(GreeterServer::new(MyGreeter::default()))
    }
}

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
