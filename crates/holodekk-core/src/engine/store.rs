use async_trait::async_trait;

use crate::Result;

use super::{Image, ImageKind};

#[async_trait]
pub trait ImageStore {
    type Image: Image;

    async fn subroutine_images(&self) -> Result<Vec<Self::Image>>;
    async fn application_images(&self) -> Result<Vec<Self::Image>>;
    async fn image_exists(&self, kind: ImageKind, name: &str) -> Result<bool>;
}
