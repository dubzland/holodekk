use async_trait::async_trait;

use crate::holodekk::Application;

use crate::Result;

use super::Image;

#[async_trait]
pub trait ImageBuilder {
    type Image: Image;

    async fn build_subroutine(&self, name: &str, tag: &str, data: Vec<u8>) -> Result<Self::Image>;
    async fn build_application(
        &self,
        application: &Application<Self::Image>,
        // bytes: &'static [u8],
        bytes: Vec<u8>,
    ) -> Result<Self::Image>;
}
