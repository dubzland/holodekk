pub mod docker;

use std::sync::RwLockReadGuard;

use async_trait::async_trait;

use serde::Serialize;

use crate::holodekk::Application;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum ImageKind {
    Subroutine,
    Service,
    Application,
}

pub trait ImageTag {
    fn name(&self) -> &str;
}

pub trait Image {
    type Tag: ImageTag;

    fn name(&self) -> &str;
    fn kind(&self) -> &ImageKind;
    fn tags(&self) -> RwLockReadGuard<'_, Vec<Self::Tag>>;
}

#[async_trait]
pub trait ImageBuilder {
    type Image: Image;

    async fn build_subroutine(
        &self,
        name: &str,
        tag: &str,
        data: Vec<u8>,
    ) -> crate::Result<Self::Image>;
    async fn build_application(
        &self,
        application: &Application<Self::Image>,
        bytes: Vec<u8>,
    ) -> crate::Result<Self::Image>;
}

#[async_trait]
pub trait ImageStore {
    type Image: Image;

    async fn subroutine_images(&self) -> crate::Result<Vec<Self::Image>>;
    async fn application_images(&self) -> crate::Result<Vec<Self::Image>>;
    async fn image_exists(&self, kind: ImageKind, name: &str) -> crate::Result<bool>;
}

#[async_trait]
pub trait Engine<T: Image> {
    async fn image_builder(&self) -> crate::Result<&dyn ImageBuilder<Image = T>>;
    async fn image_store(&self) -> crate::Result<&dyn ImageStore<Image = T>>;
}
