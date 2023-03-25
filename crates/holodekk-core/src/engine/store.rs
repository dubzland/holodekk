use async_trait::async_trait;

use crate::subroutine::Subroutine;
use crate::Result;

use super::{Image, ImageTag};

#[async_trait]
pub trait ImageStore<I, T>
where
    I: Image<T>,
    T: ImageTag,
{
    async fn subroutine_images(&self) -> Result<Vec<I>>;
    async fn application_images(&self) -> Result<Vec<I>>;
    async fn application_image_exists(&self, subroutine: &Subroutine) -> Result<bool>;
}
