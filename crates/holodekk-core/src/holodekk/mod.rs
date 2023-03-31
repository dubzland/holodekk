use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use log::warn;

use uuid::Uuid;

pub use holodekk_projector::projector::{Projector, ProjectorHandle};
pub mod subroutine;

// use crate::engine::{docker::Docker, Engine};
use crate::{Error, Result};

#[derive(Clone, Debug)]
pub struct Holodekk {
    _engine_type: String,
    projectors: Arc<RwLock<HashMap<Uuid, Projector>>>,
}

impl Holodekk {
    pub fn new(engine_type: &str) -> Self {
        Self {
            _engine_type: engine_type.to_string(),
            projectors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn projector_for_namespace(&self, namespace: &str) -> Result<ProjectorHandle> {
        let projectors = self.projectors.read().unwrap();
        if let Some((_, projector)) = projectors.iter().find(|(_, p)| p.namespace().eq(namespace)) {
            println!("Returning an existing projector: {}", projector.handle());
            Ok(projector.handle())
        } else {
            println!("Spawning a new projector");
            // Spawn a projector
            let root = PathBuf::from("/tmp/holodekk/projector/local");
            let projector = Projector::spawn("local", &root, None, None)?;
            let handle = projector.handle();
            drop(projectors);
            let mut projectors = self.projectors.write().unwrap();
            projectors.insert(*projector.id(), projector);

            Ok(handle)
        }
    }

    pub fn stop_projector(&self, handle: ProjectorHandle) -> Result<()> {
        let mut projectors = self.projectors.write().unwrap();
        if let Some(projector) = projectors.remove(&handle.id) {
            projector.stop()?;
            Ok(())
        } else {
            Err(Error::InvalidProjector {
                id: handle.id,
                namespace: handle.namespace,
            })
        }
    }

    pub fn stop(&self) -> Result<()> {
        let handles: Vec<ProjectorHandle> = self
            .projectors
            .read()
            .unwrap()
            .values()
            .map(|p| p.handle())
            .collect();
        for handle in handles.into_iter() {
            if let Some(err) = self.stop_projector(handle).err() {
                warn!("Failed to stop projector: {}", err);
            }
        }
        Ok(())
    }
}
