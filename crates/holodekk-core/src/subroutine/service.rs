use crate::engine::{Image, ImageStore, ImageTag};
use crate::errors::Result;

/// Manages both images and running instances of subroutines.
///
/// Responsible for building and maintaining container images (via an
/// [ImageStore](crate::engine::ImageStore)).
pub struct Service<'a, I, T> {
    store: &'a dyn ImageStore<I, T>,
}

impl<'a, I, T> Service<'a, I, T>
where
    I: Image<T>,
    T: ImageTag,
{
    pub fn new(store: &'a dyn ImageStore<I, T>) -> Self {
        Self { store }
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
    ///     let store = docker::Store::new();
    ///     let subroutines = subroutine::Service::new(&store);
    ///     let images = subroutines.images().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn images(&self) -> Result<Vec<I>> {
        self.store.subroutine_images().await
    }
}
