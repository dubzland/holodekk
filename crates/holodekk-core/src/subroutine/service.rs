use crate::engine::{Engine, Image, ImageTag};
use crate::errors::Result;

/// Manages both images and running instances of subroutines.
///
/// Responsible for building and maintaining container images (via an
/// [ImageStore](crate::engine::ImageStore)).
pub struct Service<'a, I, T> {
    engine: &'a dyn Engine<I, T>,
}

impl<'a, I, T> Service<'a, I, T>
where
    I: Image<T>,
    T: ImageTag,
{
    pub fn new(engine: &'a dyn Engine<I, T>) -> Self {
        Self { engine }
    }

    /// Returns the available subroutine images.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekk_core::{engine::docker, subroutine, Result};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let docker = docker::Service::new();
    ///     let subroutines = subroutine::Service::new(&docker);
    ///     let images = subroutines.images().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn images(&self) -> Result<Vec<I>> {
        self.engine.subroutine_images().await
    }
}
