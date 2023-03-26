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

use holodekk_core::engine::{Image, ImageBuilder, ImageStore};

pub struct Projector<T: Image> {
    store: Box<dyn ImageStore<Image = T>>,
    builder: Box<dyn ImageBuilder<Image = T>>,
}

impl<T: Image> Projector<T> {
    pub fn new(
        store: Box<dyn ImageStore<Image = T>>,
        builder: Box<dyn ImageBuilder<Image = T>>,
    ) -> Self {
        Self { store, builder }
    }

    pub fn store(&self) -> &dyn ImageStore<Image = T> {
        self.store.as_ref()
    }

    // pub fn builder(&self) -> &Box<dyn ImageBuilder<Image = T>> {
    pub fn builder(&self) -> &dyn ImageBuilder<Image = T> {
        self.builder.as_ref()
    }

    pub fn build() -> ProjectorBuilder<T> {
        ProjectorBuilder::new()
    }
}
