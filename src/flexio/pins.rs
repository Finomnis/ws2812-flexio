use imxrt_hal as hal;

use hal::iomuxc::{self, flexio::Pin};

use paste::paste;

/// The pins to use for WS2812.
pub trait Pins<const N: u8> {
    /// Configures the pins.
    ///
    /// This is not intended to be called by the user;
    /// it will be used inside of the driver.
    fn configure(&mut self);
}

macro_rules! impl_pins {
    ($($n:literal)+) => {
        paste! {
            impl<const N: u8, $([<P $n>]: Pin<N>),+> Pins<N> for ($([<P $n>]),+,) {
                fn configure(&mut self) {
                    $(
                        iomuxc::flexio::prepare(&mut self.$n);
                    )+
                }
            }
        }
    };
}

impl_pins!(0);
impl_pins!(0 1);
impl_pins!(0 1 2);
impl_pins!(0 1 2 3);
impl_pins!(0 1 2 3 4);
impl_pins!(0 1 2 3 4 5);
impl_pins!(0 1 2 3 4 5 6);
impl_pins!(0 1 2 3 4 5 6 7);
impl_pins!(0 1 2 3 4 5 6 7 8);
impl_pins!(0 1 2 3 4 5 6 7 8 9);
impl_pins!(0 1 2 3 4 5 6 7 8 9 10);
impl_pins!(0 1 2 3 4 5 6 7 8 9 10 11);
impl_pins!(0 1 2 3 4 5 6 7 8 9 10 11 12);
impl_pins!(0 1 2 3 4 5 6 7 8 9 10 11 12 13);
impl_pins!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14);
impl_pins!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15);
