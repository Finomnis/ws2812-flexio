/// A pixel that can be rendered with this library.
pub trait Pixel<const N: usize> {
    /// Return the raw bytes that should be sent to the LED strip.
    ///
    /// IMPORTANT: Be aware that LED strips are GRB encoded.
    fn get_ws2812_bytes(&self) -> [u8; N];
}

// Raw RGB data.
impl Pixel<3> for [u8; 3] {
    fn get_ws2812_bytes(&self) -> [u8; 3] {
        // Neopixel strips want GRB data
        [self[1], self[0], self[2]]
    }
}

// Raw RGBW data.
impl Pixel<4> for [u8; 4] {
    fn get_ws2812_bytes(&self) -> [u8; 4] {
        *self
    }
}

impl Pixel<3> for palette::LinSrgb<u8> {
    fn get_ws2812_bytes(&self) -> [u8; 3] {
        [self.green, self.red, self.blue]
    }
}

impl<'a, P, const N: usize> Pixel<N> for &'a P
where
    P: Pixel<N>,
{
    fn get_ws2812_bytes(&self) -> [u8; N] {
        (*self).get_ws2812_bytes()
    }
}
