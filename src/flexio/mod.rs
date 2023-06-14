use imxrt_hal as hal;
use imxrt_ral as ral;

use hal::iomuxc::{self, flexio::Pin};

use ral::{flexio, Valid};

/// The pins to use for WS2812.
pub trait Pins<const N: u8, const P: u8> {
    /// Configures the pins.
    ///
    /// This is not intended to be called by the user;
    /// it will be used inside of the driver.
    fn configure(&mut self);
}

impl<const N: u8, P: Pin<N>> Pins<N, 1> for P {
    fn configure(&mut self) {
        iomuxc::flexio::prepare(self);
    }
}

impl<const N: u8, P1: Pin<N>, P2: Pin<N>> Pins<N, 2> for (P1, P2) {
    fn configure(&mut self) {
        iomuxc::flexio::prepare(&mut self.0);
        iomuxc::flexio::prepare(&mut self.1);
    }
}

/// A WS2812 Neopixel LED Strip driver based on the i.MX RT FlexIO module
pub struct Ws2812Driver<const N: u8, const P: u8, PINS: Pins<N, P>>
where
    flexio::Instance<N>: Valid,
{
    flexio: flexio::Instance<N>,
    _pins: PINS,
}

impl<const N: u8, const P: u8, PINS: Pins<N, P>> Ws2812Driver<N, P, PINS>
where
    flexio::Instance<N>: Valid,
{
    /// Initializes the FlexIO driver
    pub fn init(ccm: &mut ral::ccm::CCM, flexio: flexio::Instance<N>, mut pins: PINS) -> Self {
        pins.configure();
        Self {
            flexio,
            _pins: pins,
        }
    }

    /// A dummy function for development purposes
    pub fn dummy_write(&mut self) {}
}
