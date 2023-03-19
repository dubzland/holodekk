pub mod docker;

use async_trait::async_trait;

use bytes::Bytes;

#[derive(Clone, Debug)]
pub enum ImageDefKind {
    Subroutine,
    Service,
    Application
}

#[derive(Debug)]
pub struct ImageDef {
    pub name: String,
    pub tag: String,
    pub id: Option<String>,
    pub kind: ImageDefKind,
}

#[derive(Debug)]
pub enum Error {
    BuildFailed
}

pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait Engine {
    async fn build_image(&self, def: &mut ImageDef, rootfs: &Bytes) -> Result<()>;
    async fn list_images(&self, kind: ImageDefKind) -> Vec<ImageDef>;
}
