use std::cell::Ref;

use serde::Serialize;

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
