use std::cell::Ref;

use async_trait::async_trait;

use serde::Serialize;

use crate::subroutine::Subroutine;
use crate::Result;

#[derive(Clone, Debug)]
pub enum ContainerKind {}

pub trait Container<I, T>
where
    I: Image<T>,
    T: ImageTag,
{
    fn id(&self) -> &str;
    fn kind(&self) -> &ImageKind;
    fn image(&self) -> &I;
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum ImageKind {
    Subroutine,
    Service,
    Application,
}

pub trait ImageTag {
    fn name(&self) -> &str;
}

pub trait Image<T>
where
    T: ImageTag,
{
    fn name(&self) -> &str;
    fn kind(&self) -> &ImageKind;
    fn tags(&self) -> Ref<'_, Vec<T>>;
}

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

#[async_trait]
pub trait ImageBuilder<I, T>
where
    I: Image<T>,
    T: ImageTag,
{
    async fn build_subroutine(&self, name: &str, tag: &str, data: Vec<u8>) -> Result<I>;
    async fn build_application(&self, subroutine: &Subroutine) -> Result<I>;
}
