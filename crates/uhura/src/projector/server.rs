use std::fmt;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

use holodekk_utils::{ApiListenerKind, ApiServer};

use super::builder::ProjectorServerBuilder;
use crate::api::server::UhuraApi;
use holodekk_projector::{api::server::ApplicationsService, Result};

// use crate::Result;

pub struct ProjectorServer {
    namespace: String,
    runtime: tokio::runtime::Runtime,
    thread_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
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
            thread_handle: Arc::new(RwLock::new(None)),
            uhura_api: Arc::new(uhura_api),
            projector_api: Arc::new(projector_api),
        }
    }

    pub fn uhura_listener(&self) -> ApiListenerKind {
        self.uhura_api.listener()
    }

    pub fn start(&self) -> Result<()> {
        let _guard = self.runtime.enter();
        let handle = tokio::runtime::Handle::current();

        let uhura_api = self.uhura_api.clone();
        let projector_api = self.projector_api.clone();

        let thread_handle = std::thread::spawn(move || {
            let _guard = handle.enter();

            let uhura_handle = uhura_api.start();
            let projector_handle = projector_api.start();

            handle.block_on(async move {
                uhura_handle.await.unwrap().unwrap();
                projector_handle.await.unwrap().unwrap();
            });
        });
        self.thread_handle.write().unwrap().replace(thread_handle);
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        if self.thread_handle.read().unwrap().is_some() {
            let thread_handle = self.thread_handle.write().unwrap().take().unwrap();
            self.uhura_api.stop();
            self.projector_api.stop();
            thread_handle.join().unwrap();
        }
        // let _guard = self.runtime.enter();
        // let runtime = tokio::runtime::Handle::current();
        // let uhura_api = self.uhura_api.clone();
        // let projector_api = self.projector_api.clone();
        // runtime.block_on(async {
        //     uhura_api.stop().await.unwrap();
        //     projector_api.stop().await.unwrap();
        // });
        // let thread_handle = self.thread_handle.borrow_mut().take().unwrap();
        // thread_handle.join().unwrap();
        Ok(())
    }
}

impl Drop for ProjectorServer {
    fn drop(&mut self) {
        // call stop to be safe
        self.stop().unwrap();
    }
}
