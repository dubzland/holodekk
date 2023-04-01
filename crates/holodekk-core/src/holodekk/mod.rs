use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use log::{debug, warn};

use uuid::Uuid;

pub use holodekk_projector::projector::{Projector, ProjectorHandle};
pub mod subroutine;

use crate::{Error, Result};

// #[derive(Debug)]
#[derive(Clone, Debug)]
pub struct Holodekk {
    root_path: PathBuf,
    bin_path: PathBuf,
    projectors: Arc<RwLock<HashMap<Uuid, Projector>>>,
}

impl Holodekk {
    pub fn new<P: AsRef<Path>>(root: P, bin: P) -> Self {
        Self {
            root_path: root.as_ref().to_path_buf(),
            bin_path: bin.as_ref().to_path_buf(),
            projectors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn init(&self) -> std::io::Result<()> {
        // ensure the root path exists
        if !self.root_path.exists() {
            fs::create_dir_all(&self.root_path)?;
        }
        Ok(())
    }

    /// Returns a handle for the given namespace.
    ///
    /// If a projector is not currently running for the specified namespace, one will be started.
    ///
    /// # Arguments
    ///
    /// `namespace` - desired namespace for the projector.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use holodekk::Holodekk;
    ///
    /// let holodekk = Holodekk::new("/tmp", "/usr/local/bin");
    /// holodekk.init().unwrap();
    /// let projector = holodekk.projector_for_namespace("local").unwrap();
    /// ```
    pub fn projector_for_namespace(&self, namespace: &str) -> Result<ProjectorHandle> {
        debug!("Inside projector_for_namespace()");
        let projectors = self.projectors.read().unwrap();
        if let Some((_, projector)) = projectors.iter().find(|(_, p)| p.namespace().eq(namespace)) {
            Ok(projector.handle())
        } else {
            // Spawn a projector
            let mut projector_root = self.root_path.clone();
            projector_root.push(namespace);
            let projector =
                Projector::spawn(namespace, &projector_root, &self.bin_path, None, None)?;
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
            // It will die as soon as everyone releases it.
            drop(projector);
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
