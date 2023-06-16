use snafu::prelude::*;

/// Errors of the [WS2812Driver::init] function
#[derive(Debug, Snafu)]
pub enum WS2812InitError {
    /// The peripheral does not have enough IO Pins.
    //#[snafu(display("The peripheral does not have enough IO Pins."))]
    NotEnoughPins,
}
