pub mod docker;
mod images;
pub use images::*;

pub trait Engine<I, T>: ImageStore<I, T> + ImageBuilder<I, T>
where
    I: Image<T>,
    T: ImageTag
{}
