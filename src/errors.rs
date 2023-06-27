use snafu::prelude::*;

/// Errors of the [WS2812Driver::init] function
#[derive(Debug, Snafu)]
pub enum WS2812InitError {
    /// The peripheral does not have enough IO pins.
    NotEnoughPins,
    /// Unable to find 4 free FlexIO pins in a row; required for how the shifter is set up.
    NeedFourConsecutiveInternalPins,
    /// The peripheral does not have enough shifters for the given amount of pins.
    NotEnoughShifters,
    /// The peripheral does not have enough timers for the given amount of pins.
    NotEnoughTimers,
}
