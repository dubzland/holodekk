use std::cell::RefCell;
use std::fmt;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

mod builder;
pub use builder::ProjectorServerBuilder;
mod services;
pub use services::Service;

use crate::Result;

pub struct ProjectorServer {
    namespace: String,
    runtime: tokio::runtime::Runtime,
    thread_handle: RefCell<Option<JoinHandle<()>>>,
    admin_service: Arc<Service>,
    projector_service: Arc<Service>,
}

impl fmt::Debug for ProjectorServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Projector")
            .field("namespace", &self.namespace)
            .finish()
    }
}

impl ProjectorServer {
    pub fn build() -> ProjectorServerBuilder {
        ProjectorServerBuilder::new()
    }

    pub fn new(namespace: &str, admin_service: Service, projector_service: Service) -> Self {
        Self {
            namespace: namespace.to_string(),
            runtime: tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"),
            thread_handle: RefCell::new(None),
            admin_service: Arc::new(admin_service),
            projector_service: Arc::new(projector_service),
        }
    }

    pub fn start(&self) -> Result<()> {
        let _guard = self.runtime.enter();
        let handle = tokio::runtime::Handle::current();
        let admin_service = self.admin_service.clone();
        let projector_service = self.projector_service.clone();
        let thread_handle = thread::spawn(move || {
            handle.block_on(async move {
                let admin_handle = admin_service.start().unwrap();
                let projector_handle = projector_service.start().unwrap();

                admin_handle.await.unwrap().unwrap();
                projector_handle.await.unwrap().unwrap();
            });
        });
        self.thread_handle.borrow_mut().replace(thread_handle);
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        let _guard = self.runtime.enter();
        let runtime = tokio::runtime::Handle::current();
        let admin_service = self.admin_service.clone();
        let projector_service = self.projector_service.clone();
        runtime.block_on(async {
            admin_service.stop().await.unwrap();
            projector_service.stop().await.unwrap();
        });
        let thread_handle = self.thread_handle.borrow_mut().take().unwrap();
        thread_handle.join().unwrap();
        Ok(())
    }
}
