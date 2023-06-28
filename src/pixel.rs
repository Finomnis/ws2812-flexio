/// A pixel that can be rendered with this library.
pub trait Pixel {
    /// The return type of the [into_ws2812_bytes()](Pixel::into_ws2812_bytes) function.
    type BytesIter: Iterator<Item = u8>;

    /// Return the raw bytes that should be sent to the LED strip.
    ///
    /// IMPORTANT: Be aware that WS2812 strips are GRB encoded.
    fn into_ws2812_bytes(self) -> Self::BytesIter;
}

/// Raw RGB data.
impl Pixel for [u8; 3] {
    type BytesIter = core::array::IntoIter<u8, 3>;

    fn into_ws2812_bytes(self) -> Self::BytesIter {
        // Neopixel strips want GRB data
        [self[1], self[0], self[2]].into_iter()
    }
}

/// Raw RGBW data.
impl Pixel for [u8; 4] {
    type BytesIter = core::array::IntoIter<u8, 4>;

    fn into_ws2812_bytes(self) -> Self::BytesIter {
        self.into_iter()
    }
}

/// 8-bit Linear sRGB, which is the color space
/// most NeoPixel strips are in.
///
/// Be aware that this differs from normal,
/// gamma-corrected sRGB. A conversion has to take place.
///
/// More info can be found in the documentation of the
/// [palette] crate.
impl Pixel for palette::LinSrgb<u8> {
    type BytesIter = core::array::IntoIter<u8, 3>;

    fn into_ws2812_bytes(self) -> Self::BytesIter {
        [self.green, self.red, self.blue].into_iter()
    }
}

impl<'a, P> Pixel for &'a P
where
    P: Pixel + Clone,
{
    type BytesIter = <P as Pixel>::BytesIter;
    fn into_ws2812_bytes(self) -> Self::BytesIter {
        self.clone().into_ws2812_bytes()
    }
}
