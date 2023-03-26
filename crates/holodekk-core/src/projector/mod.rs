//! The Projector implementation for the Holodekk.
//!
//! The projector is the glue between the Holodekk platform and the subroutines it runs.
//! Subroutines make requests of the Projector, and the Projector keeps them up to date with the
//! current state of the system.

pub mod client;
mod error;
pub use error::{ProjectorError, Result};

mod builder;
pub use builder::ProjectorBuilder;

pub mod server;

use crate::engine::Engine;

pub struct Projector {
    engine: Box<dyn Engine>,
}

impl Projector {
    pub fn new(engine: Box<dyn Engine>) -> Self {
        Self { engine }
    }

    pub fn engine(&self) -> &dyn Engine {
        self.engine.as_ref()
    }

    pub fn build() -> ProjectorBuilder {
        ProjectorBuilder::new()
    }
}
