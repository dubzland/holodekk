// use std::cell::RefCell;
use std::fmt;
use std::sync::Arc;
// use std::thread::JoinHandle;

use holodekk_utils::ApiServer;

use super::builder::ProjectorServerBuilder;
use crate::api::server::UhuraApi;
use holodekk_projector::{api::server::ApplicationsService, Result};

// use crate::Result;

pub struct ProjectorServer {
    namespace: String,
    runtime: tokio::runtime::Runtime,
    // thread_handle: RefCell<Option<JoinHandle<()>>>,
    uhura_api: Arc<ApiServer<UhuraApi>>,
    projector_api: Arc<ApiServer<ApplicationsService>>,
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

    pub fn new(
        namespace: &str,
        uhura_api: ApiServer<UhuraApi>,
        projector_api: ApiServer<ApplicationsService>,
    ) -> Self {
        Self {
            namespace: namespace.to_string(),
            runtime: tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"),
            // thread_handle: RefCell::new(None),
            uhura_api: Arc::new(uhura_api),
            projector_api: Arc::new(projector_api),
        }
    }

    pub fn start(&self) -> Result<()> {
        let _guard = self.runtime.enter();
        // let uhura_api = self.uhura_api.clone();
        // let projector_api = self.projector_api.clone();
        // let thread_handle = thread::spawn(move || {
        // let _guard = handle.enter();
        self.uhura_api.start();
        self.projector_api.start();
        // handle.block_on(async move {
        //     uhura_handle.await.unwrap().unwrap();
        //     projector_handle.await.unwrap().unwrap();
        // });
        // });
        // self.thread_handle.borrow_mut().replace(thread_handle);
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        let _guard = self.runtime.enter();
        let runtime = tokio::runtime::Handle::current();
        let uhura_api = self.uhura_api.clone();
        let projector_api = self.projector_api.clone();
        runtime.block_on(async {
            uhura_api.stop().await.unwrap();
            projector_api.stop().await.unwrap();
        });
        // let thread_handle = self.thread_handle.borrow_mut().take().unwrap();
        // thread_handle.join().unwrap();
        Ok(())
    }
}
