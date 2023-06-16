use imxrt_hal as hal;
use imxrt_ral as ral;

use hal::ccm::clock_gate;
use ral::{flexio, Valid};

/// Errors the driver can cause
pub mod errors;
mod pins;

pub use pins::Pins;

/// A WS2812 Neopixel LED Strip driver based on the i.MX RT FlexIO module
pub struct Ws2812Driver<const N: u8, PINS: Pins<N>>
where
    flexio::Instance<N>: Valid,
{
    flexio: flexio::Instance<N>,
    _pins: PINS,
}

impl<const N: u8, PINS: Pins<N>> Ws2812Driver<N, PINS>
where
    flexio::Instance<N>: Valid,
{
    /// Initializes the FlexIO driver
    pub fn init(
        ccm: &mut ral::ccm::CCM,
        flexio: flexio::Instance<N>,
        mut pins: PINS,
    ) -> Result<Self, errors::WS2812InitError> {
        // Configure clocks
        clock_gate::flexio::<N>().set(ccm, clock_gate::ON);

        // Parameter check
        let (version_major, version_minor, available_feature_set) =
            ral::read_reg!(ral::flexio, flexio, VERID, MAJOR, MINOR, FEATURE);
        let (available_triggers, available_pins, available_timers, available_shifters) =
            ral::read_reg!(ral::flexio, flexio, PARAM, TRIGGER, PIN, TIMER, SHIFTER);

        if available_pins < PINS::PIN_COUNT {
            return Err(errors::WS2812InitError::NotEnoughPins);
        }
        // TODO check timers, shifters and triggers count

        // Debug infos
        log::debug!("Initializing FlexIO #{}.", N);
        log::debug!("    Version: {}.{}", version_major, version_minor);
        log::debug!("    Feature Set: {}", available_feature_set);
        log::debug!("    Peripherals:");
        log::debug!("        {} triggers", available_triggers);
        log::debug!("        {} pins", available_pins);
        log::debug!("        {} timers", available_timers);
        log::debug!("        {} shifters", available_shifters);
        log::debug!("Pin Offsets: {:?}", PINS::FLEXIO_PIN_OFFSETS);

        // Configure pins
        pins.configure();

        Ok(Self {
            flexio,
            _pins: pins,
        })
    }

    /// A dummy function for development purposes
    pub fn dummy_write(&mut self) {}
}

// TODO: Add PreparedPixels struct that contains prepared pixels.
// TODO: Add Iterator<Item = (Pixel, Pixel)> based pixel-setter function.
