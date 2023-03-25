mod builder;
pub use builder::*;
pub mod docker;
mod image;
pub use image::*;
mod store;
pub use store::*;

pub trait Engine<I, T>: ImageStore<I, T> + ImageBuilder<I, T>
where
    I: Image<T>,
    T: ImageTag,
{
}
