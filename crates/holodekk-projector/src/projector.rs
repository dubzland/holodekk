use std::future::Future;

use crate::client::ProjectorClient;
use crate::error::Result;

pub struct Projector {
    client: ProjectorClient,
    rt: Option<tokio::runtime::Runtime>,
}

impl Projector {
    pub fn new(port: u16) -> Self {
        let rt = match tokio::runtime::Handle::try_current() {
            Ok(_) => None,
            Err(_) => Some(tokio::runtime::Runtime::new().unwrap()),
        };
        let client = wait_for_future(
            rt.as_ref(),
            ProjectorClient::build().connect_tcp(port, None),
        )
        .unwrap();
        Self { client, rt }
    }

    pub fn say_hello(&self, name: String) -> Result<String> {
        wait_for_future(self.rt.as_ref(), self.client.say_hello(&name))
    }
}

fn wait_for_future<F: Future>(runtime: Option<&tokio::runtime::Runtime>, f: F) -> F::Output
where
    F: Send,
    F::Output: Send,
{
    if runtime.is_some() {
        runtime.unwrap().block_on(f)
    } else {
        let handle = tokio::runtime::Handle::current();
        let _guard = handle.enter();
        let h2 = tokio::runtime::Handle::current();
        h2.block_on(f)
    }
}
