use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, RwLock};

use log::{debug, warn};
use uuid::Uuid;

use holodekk::config::HolodekkConfig;

pub use crate::projector::{Projector, ProjectorHandle};

use crate::errors::HolodekkError;

pub type HolodekkResult<T> = std::result::Result<T, HolodekkError>;

// #[derive(Debug)]
#[derive(Clone, Debug)]
pub struct Holodekk {
    pub config: Arc<HolodekkConfig>,
    pub projectors: Arc<RwLock<HashMap<Uuid, Projector>>>,
}

impl Holodekk {
    pub fn new(config: HolodekkConfig) -> Self {
        Self {
            config: Arc::new(config),
            projectors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn init(&self) -> std::io::Result<()> {
        // ensure the root path exists
        if !self.config.root_path.exists() {
            fs::create_dir_all(&self.config.root_path)?;
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
    /// ```rust,ignore
    /// use holodekk::Holodekk;
    ///
    /// let holodekk = Holodekk::new("/var/lib/holodekk", "/usr/local/bin");
    /// holodekk.init().unwrap();
    /// let projector = holodekk.projector_for_namespace("local").unwrap();
    /// ```
    pub fn projector_for_namespace(&self, namespace: &str) -> HolodekkResult<ProjectorHandle> {
        debug!("Inside projector_for_namespace()");
        let projectors = self.projectors.read().unwrap();
        let handle = if let Some((_, projector)) =
            projectors.iter().find(|(_, p)| p.namespace.eq(namespace))
        {
            projector.handle()
        } else {
            // Spawn a projector
            let mut projector_root = self.config.root_path.clone();
            projector_root.push(namespace);
            let projector = Projector::spawn(
                &self.config.fleet,
                namespace,
                &projector_root,
                &self.config.bin_path,
                None,
                None,
            )?;
            drop(projectors);
            let mut projectors = self.projectors.write().unwrap();
            let handle = projector.handle();
            projectors.insert(projector.id, projector);
            handle
        };
        Ok(handle)
    }

    pub fn stop_projector(&self, id: Uuid) -> HolodekkResult<()> {
        let mut projectors = self.projectors.write().unwrap();
        if let Some(projector) = projectors.remove(&id) {
            // It will die as soon as everyone releases it.
            drop(projector);
            Ok(())
        } else {
            Err(HolodekkError::InvalidProjector { id })
        }
    }

    pub async fn stop(&self) -> HolodekkResult<()> {
        let projectors = self.projectors.read().unwrap();
        let ids = projectors.values().map(|p| p.id);
        for projector in ids {
            if let Some(err) = self.stop_projector(projector).err() {
                warn!("Failed to stop projector: {}", err);
            }
        }
        // for projector in self.projectors.read()
        // self
        //     .projectors
        //     .read()
        //     .unwrap()
        //     .values()
        //     .map(|p| {
        //     });'
        // p.handle().await)
        //     .collect();
        // for handle in handles.into_iter() {
        //     if let Some(err) = self.stop_projector(handle).err() {
        //         warn!("Failed to stop projector: {}", err);
        //     }
        // }
        Ok(())
    }
}
