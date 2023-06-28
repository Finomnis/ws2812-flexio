use crate::Pixel;

pub trait PixelStreamRef {
    fn next(&mut self) -> Option<u8>;
}

pub struct PixelStream<P, I>
where
    P: Pixel,
    I: Iterator<Item = P>,
{
    pixel_stream: I,
}

impl<P, I> PixelStreamRef for PixelStream<P, I>
where
    P: Pixel,
    I: Iterator<Item = P>,
{
    fn next(&mut self) -> Option<u8> {
        todo!()
    }
}

/// Converts an iterator of pixels into a pixel stream, usable by the driver's `write` function.
pub trait IntoPixelStream {
    /// The pixel type.
    type Pixel: Pixel;
    /// The pixel iterator type.
    type PixelIter: Iterator<Item = Self::Pixel>;

    /// Converts the current object into a pixel stream.
    fn into_pixel_stream(self) -> PixelStream<Self::Pixel, Self::PixelIter>;
}

impl<T> IntoPixelStream for T
where
    T: IntoIterator,
    <T as IntoIterator>::Item: Pixel,
{
    type Pixel = <T as IntoIterator>::Item;
    type PixelIter = <T as IntoIterator>::IntoIter;

    fn into_pixel_stream(self) -> PixelStream<Self::Pixel, Self::PixelIter> {
        todo!();
        // PixelStream {
        //     pixel_stream: self.into_iter().map(Pixel::get_ws2812_bytes).flatten(),
        // }
    }
}
