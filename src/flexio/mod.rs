use imxrt_ral as ral;

use ral::{flexio, Valid};

mod driver;
mod driver_builder;
mod pins;
mod pixel;
mod prepared_pixels;

/// Errors the driver can cause
pub mod errors;

pub use pins::Pins;
pub use pixel::Pixel;
pub use prepared_pixels::{PreparedPixels, PreparedPixelsRef};

/// A WS2812 Neopixel LED Strip driver based on the i.MX RT FlexIO module
pub struct Ws2812Driver<const N: u8, PINS: Pins<N>>
where
    flexio::Instance<N>: Valid,
{
    flexio: flexio::Instance<N>,
    _pins: PINS,
}

// TODO: Add PreparedPixels struct that contains prepared pixels.
// TODO: Add Iterator<Item = (Pixel, Pixel)> based pixel-setter function.
