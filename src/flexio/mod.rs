use imxrt_ral as ral;

use ral::{flexio, Valid};

mod dma;
mod driver;
mod flexio_configurator;
mod interleaved_pixels;
mod preprocessed_pixels;

use crate::Pins;

pub use preprocessed_pixels::PreprocessedPixels;

/// A WS2812 Neopixel LED Strip driver based on the i.MX RT FlexIO module
pub struct WS2812Driver<const N: u8, const L: usize, PINS: Pins<N, L>>
where
    flexio::Instance<N>: Valid,
{
    flexio: flexio::Instance<N>,
    _pins: PINS,
}
