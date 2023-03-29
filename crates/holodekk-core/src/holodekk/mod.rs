// use std::cell::{Ref, RefCell};
// use std::collections::HashMap;
// use std::ops::Deref;

pub mod subroutine;

// use crate::engine::{docker::Docker, Engine};
// use crate::errors::{Error, Result};

pub struct Holodekk {
    engine_type: String,
    // projectors: RefCell<HashMap<String, Projector>>,
}

impl Holodekk {
    pub fn new(engine_type: &str) -> Self {
        Self {
            engine_type: engine_type.to_string(),
            // projectors: RefCell::new(HashMap::new()),
        }
    }

    //     pub fn projector(&self, namespace: &str) -> Result<impl Deref<Target = Projector> + '_> {
    //         if !self.projectors.borrow().contains_key(namespace) {
    //             let projector = self.create_projector(namespace)?;
    //             self.projectors
    //                 .borrow_mut()
    //                 .insert(namespace.to_string(), projector);
    //         }

    //         Ok(Ref::map(self.projectors.borrow(), |projectors| {
    //             projectors.get(namespace).unwrap()
    //         }))
    //     }

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
