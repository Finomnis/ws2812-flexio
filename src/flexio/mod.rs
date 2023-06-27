use imxrt_ral as ral;

use ral::{flexio, Valid};

mod dma;
mod driver;
mod driver_builder;
/// Errors the driver can cause
pub mod errors;
mod pins;

pub use pins::Pins;

/// A WS2812 Neopixel LED Strip driver based on the i.MX RT FlexIO module
pub struct Ws2812Driver<const N: u8, const L: usize, PINS: Pins<N, L>>
where
    flexio::Instance<N>: Valid,
{
    flexio: flexio::Instance<N>,
    _pins: PINS,
}
