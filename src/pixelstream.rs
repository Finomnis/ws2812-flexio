use crate::pixel::Pixel;

pub trait PixelStreamRef {
    fn next(&mut self) -> Option<u8>;
}

pub struct PixelStream<P, I>
where
    P: Pixel,
    I: Iterator<Item = P>,
{
    pixel_stream: I,
    bytes_iter: Option<P::BytesIter>,
    finished: bool,
}

impl<I, P> PixelStream<P, I>
where
    P: Pixel,
    I: Iterator<Item = P>,
{
    fn new(pixel_stream: I) -> Self {
        Self {
            pixel_stream,
            bytes_iter: None,
            finished: false,
        }
    }
}

impl<I, P> PixelStreamRef for PixelStream<P, I>
where
    P: Pixel,
    I: Iterator<Item = P>,
{
    fn next(&mut self) -> Option<u8> {
        loop {
            if self.finished {
                return None;
            }

            if self.bytes_iter.is_none() {
                self.bytes_iter = self.pixel_stream.next().map(|p| p.into_ws2812_bytes());
            }

            if let Some(bytes_iter) = self.bytes_iter.as_mut() {
                if let Some(byte) = bytes_iter.next() {
                    return Some(byte);
                } else {
                    self.bytes_iter = None;
                }
            } else {
                self.finished = true;
            }
        }
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
        PixelStream::new(self.into_iter())
    }
}
