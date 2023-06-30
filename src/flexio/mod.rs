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

/// The result of [WS2812Driver::write_dma()][WS2812Driver::write_dma].
pub struct WriteDmaResult<R> {
    /// The result of the concurrent function
    pub result: R,
    /// True if the concurrent function took longer than writing the
    /// data to the LED strips. This might indicate a render lag.
    pub lagged: bool,
}
