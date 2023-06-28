use crate::Pixel;

pub struct PixelStream {
    // pixel_stream: &'a mut dyn Iterator<Item =
}

pub trait IntoPixelStream {}

impl<T> IntoPixelStream for T
where
    T: IntoIterator,
    <T as IntoIterator>::Item: Pixel,
{
}
