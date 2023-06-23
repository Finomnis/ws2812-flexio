use snafu::prelude::*;

/// Errors of the [WS2812Driver::init] function
#[derive(Debug, Snafu)]
pub enum WS2812InitError {
    /// The peripheral does not have enough IO pins.
    NotEnoughPins,
    /// The peripheral does not have enough shifters for the given amount of pins.
    NotEnoughShifters,
    /// The peripheral does not have enough timers for the given amount of pins.
    NotEnoughTimers,
}
