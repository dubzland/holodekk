use async_trait::async_trait;

use crate::subroutine::Subroutine;
use crate::Result;

use super::{Image, ImageTag};

#[async_trait]
pub trait ImageBuilder<I, T>
where
    I: Image<T>,
    T: ImageTag,
{
    async fn build_subroutine(&self, name: &str, tag: &str, data: Vec<u8>) -> Result<I>;
    async fn build_application(&self, subroutine: &Subroutine) -> Result<I>;
}
