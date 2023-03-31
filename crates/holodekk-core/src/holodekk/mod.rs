use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use log::warn;

use uuid::Uuid;

pub use holodekk_projector::projector::{Projector, ProjectorHandle};
pub mod subroutine;

// use crate::engine::{docker::Docker, Engine};
use crate::errors::{Error, Result};

pub struct Holodekk {
    _engine_type: String,
    projectors: RefCell<HashMap<Uuid, Projector>>,
}

impl Holodekk {
    pub fn new(engine_type: &str) -> Self {
        Self {
            _engine_type: engine_type.to_string(),
            projectors: RefCell::new(HashMap::new()),
        }
    }

    pub fn projector_for_namespace(&self, namespace: &str) -> Result<ProjectorHandle> {
        let projectors = self.projectors.borrow();
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
            let mut projectors = self.projectors.borrow_mut();
            projectors.insert(projector.id().clone(), projector);

            Ok(handle)
        }
    }

    pub fn stop_projector(&self, handle: ProjectorHandle) -> Result<()> {
        let mut projectors = self.projectors.borrow_mut();
        if let Some(projector) = projectors.remove(&handle.id) {
            projector.stop()?;
            Ok(())
        } else {
            Err(Error::InvalidProjector(handle.clone()))
        }
    }

    pub fn stop(&self) -> Result<()> {
        let handles: Vec<ProjectorHandle> = self
            .projectors
            .borrow()
            .values()
            .map(|p| p.handle().clone())
            .collect();
        for handle in handles.into_iter() {
            if let Some(err) = self.stop_projector(handle).err() {
                warn!("Failed to stop projector: {}", err);
            }
        }
        Ok(())
    }

    //     fn create_projector(&self, namespace: &str) -> Result<Projector> {
    //         let engine = self.create_engine(&self.engine_type)?;
    //         let projector = Projector::new(namespace, engine);
    //         Ok(projector)
    //     }

    //     fn create_engine(&self, engine_type: &str) -> Result<Box<dyn Engine>> {
    //         match engine_type {
    //             "docker" => Ok(Box::new(Docker::new())),
    //             _ => Err(Error::InvalidEngine(engine_type.to_string())),
    //         }
    //     }
}
