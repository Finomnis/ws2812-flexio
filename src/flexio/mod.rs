use imxrt_hal as hal;
use imxrt_ral as ral;

use hal::ccm::clock_gate;
use ral::{flexio, Valid};

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
    pub fn init(ccm: &mut ral::ccm::CCM, flexio: flexio::Instance<N>, mut pins: PINS) -> Self {
        // Configure clocks
        clock_gate::flexio::<N>().set(ccm, clock_gate::ON);

        // Configure pins
        pins.configure();

        // Debug infos
        if log::log_enabled!(log::Level::Debug) {
            let (major, minor, feature) =
                ral::read_reg!(ral::flexio, flexio, VERID, MAJOR, MINOR, FEATURE);
            let (trigger, pin, timer, shifter) =
                ral::read_reg!(ral::flexio, flexio, PARAM, TRIGGER, PIN, TIMER, SHIFTER);

            log::debug!("Initializing FlexIO #{}.", N);
            log::debug!("    Version: {}.{}", major, minor);
            log::debug!("    Feature Set: {}", feature);
            log::debug!("    Peripherals:");
            log::debug!("        {} triggers", trigger);
            log::debug!("        {} pins", pin);
            log::debug!("        {} timers", timer);
            log::debug!("        {} shifters", shifter);
        }

        Self {
            flexio,
            _pins: pins,
        }
    }

    /// A dummy function for development purposes
    pub fn dummy_write(&mut self) {}
}

// TODO: Add PreparedPixels struct that contains prepared pixels.
// TODO: Add Iterator<Item = (Pixel, Pixel)> based pixel-setter function.
