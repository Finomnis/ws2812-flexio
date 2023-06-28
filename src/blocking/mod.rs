use imxrt_ral as ral;

use ral::{flexio, Valid};

mod driver;
mod interleaved_pixels;

use crate::Pins;

/// A WS2812 Neopixel LED Strip driver based on the i.MX RT FlexIO module
pub struct WS2812Driver<const N: u8, const L: usize, PINS: Pins<N, L>>
where
    flexio::Instance<N>: Valid,
{
    flexio: flexio::Instance<N>,
    _pins: PINS,
}
