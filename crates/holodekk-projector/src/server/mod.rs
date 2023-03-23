mod handle;
pub use handle::ProjectorHandle;

mod runtime;

mod server;
pub use server::ProjectorServer;

use hello_world::{greeter_server::Greeter, HelloReply, HelloRequest};

use tonic::{Request, Response, Status};

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
