/// A pixel that can be rendered with this library.
pub trait Pixel {
    /// The return type of the bytes iter
    type BytesIter: Iterator<Item = u8>;

    /// Return the raw bytes that should be sent to the LED strip.
    ///
    /// IMPORTANT: Be aware that LED strips are GRB encoded.
    fn get_ws2812_bytes(&self) -> Self::BytesIter;
}

// Raw RGB data.
impl Pixel for [u8; 3] {
    type BytesIter = PixelBytes<3>;

    fn get_ws2812_bytes(&self) -> PixelBytes<3> {
        // Neopixel strips want GRB data
        PixelBytes::new([self[1], self[0], self[2]])
    }
}

// Raw RGBW data.
impl Pixel for [u8; 4] {
    type BytesIter = PixelBytes<4>;

    fn get_ws2812_bytes(&self) -> PixelBytes<4> {
        PixelBytes::new(*self)
    }
}

impl Pixel for palette::LinSrgb<u8> {
    type BytesIter = PixelBytes<3>;

    fn get_ws2812_bytes(&self) -> PixelBytes<3> {
        PixelBytes::new([self.green, self.red, self.blue])
    }
}

impl<'a, P> Pixel for &'a P
where
    P: Pixel,
{
    type BytesIter = <P as Pixel>::BytesIter;
    fn get_ws2812_bytes(&self) -> Self::BytesIter {
        (*self).get_ws2812_bytes()
    }
}

/// An iterator over the WS2812 bytes of a pixel
pub struct PixelBytes<const N: usize> {
    data: [u8; N],
    iter_pos: usize,
}

impl<const N: usize> PixelBytes<N> {
    fn new(data: [u8; N]) -> Self {
        Self { data, iter_pos: 0 }
    }
}

impl<const N: usize> Iterator for PixelBytes<N> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.data.get(self.iter_pos) {
            self.iter_pos += 1;
            Some(*item)
        } else {
            None
        }
    }
}
