use std::future::Future;

pub(crate) struct ProjectorRuntime {
    rt: Option<tokio::runtime::Runtime>,
}

impl ProjectorRuntime {
    pub(crate) fn create() -> Self {
        if tokio::runtime::Handle::try_current().is_ok() {
            Self { rt: None }
        } else {
            // No active runtime found.  Try to create one.
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .thread_name("holodekk-projector")
                .enable_io()
                .build()
                .unwrap();
            Self { rt: Some(runtime) }
        }
    }

    pub(crate) fn spawn_server<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        if let Some(rt) = &self.rt {
            rt.handle().spawn(async { future.await })
        } else {
            tokio::runtime::Handle::current().spawn(future)
        }
    }

    pub(crate) fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        if let Some(rt) = &self.rt {
            rt.handle().block_on(future)
        } else {
            match tokio::runtime::Handle::try_current() {
                Ok(outer_handle) => {
                    let _guard = outer_handle.enter();
                    futures::executor::block_on(future)
                }
                Err(_) => tokio::runtime::Runtime::new().unwrap().block_on(future),
            }
        }
    }
}
