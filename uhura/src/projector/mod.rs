//! The Projector implementation for the Holodekk.
//!
//! The projector is the glue between the Holodekk platform and the subroutines it runs.
//! Subroutines make requests of the Projector, and the Projector keeps them up to date with the
//! current state of the system.
use std::cell::RefCell;
use std::fmt;

use holodekk::engine::Engine;
use holodekk_projector::AdminService;
// use holodekk_projector::ProjectorServer;
use holodekk_projector::MyGreeter;
use holodekk_projector::Result;
use holodekk_projector::{ServerHandle, ServerManager};

mod builder;

use builder::ProjectorBuilder;

pub struct Projector {
    namespace: String,
    engine: Box<dyn Engine>,
    runtime: tokio::runtime::Runtime,
    handles: RefCell<Vec<ServerHandle>>,
    manager: ServerManager,
}

impl fmt::Debug for Projector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Projector")
            .field("namespace", &self.namespace)
            .field("engine", &self.engine.name())
            .finish()
    }
}

impl Projector {
    pub fn build() -> ProjectorBuilder {
        ProjectorBuilder::new()
    }

    pub fn new(namespace: &str, engine: Box<dyn Engine>) -> Self {
        Self {
            namespace: namespace.to_string(),
            engine,
            // server: ProjectorServer::new(),
            runtime: tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"),
            handles: RefCell::new(vec![]),
            manager: ServerManager::new(),
        }
    }

    pub fn _engine(&self) -> &dyn Engine {
        self.engine.as_ref()
    }

    pub fn start_admin(&self) -> Result<u16> {
        let _guard = self.runtime.enter();
        let admin_service = AdminService::build();
        let handle = self.manager.start_tcp(admin_service, None, None)?;
        let port = handle.port();
        self.handles.borrow_mut().push(handle);
        Ok(port)
    }

    pub fn start_subroutine(&self) -> Result<u16> {
        let _guard = self.runtime.enter();
        let subroutine_service = MyGreeter::build();
        let handle = self.manager.start_tcp(subroutine_service, None, None)?;
        let port = handle.port();
        self.handles.borrow_mut().push(handle);
        Ok(port)
    }

    pub fn start(&self) -> Result<(u16, u16)> {
        let admin_port = self.start_admin()?;
        let subroutine_port = self.start_subroutine()?;
        Ok((admin_port, subroutine_port))
    }

    pub fn stop(&self) -> Result<()> {
        let _guard = self.runtime.enter();
        let runtime = tokio::runtime::Handle::current();
        for handle in self.handles.borrow().iter() {
            runtime.block_on(self.manager.stop(handle))?;
        }
        Ok(())
    }
}
