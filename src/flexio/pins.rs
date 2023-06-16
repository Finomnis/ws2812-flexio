use imxrt_hal as hal;

use hal::iomuxc::{self, flexio::Pin};

use paste::paste;

/// The pins to use for WS2812.
pub trait Pins<const N: u8> {
    /// The amount of pins this object contains.
    const PIN_COUNT: u32;

    /// Configures the pins.
    ///
    /// This is not intended to be called by the user;
    /// it will be used inside of the driver.
    fn configure(&mut self);

    /// The FlexIO pin offsets
    const FLEXIO_PIN_OFFSETS: &'static [u8];
}

macro_rules! count {
    () => (0u32);
    ( $x:tt $($xs:tt)* ) => (1u32 + count!($($xs)*));
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

                const PIN_COUNT: u32 = count!($($n)+);

                const FLEXIO_PIN_OFFSETS: &'static[u8] = &[$(
                    [<P $n>]::OFFSET
                ),+];
            }
        }
    };
}

impl_pins!(0);
impl_pins!(0 1);
impl_pins!(0 1 2 3);
impl_pins!(0 1 2 3 4 5 6 7);
impl_pins!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15);
impl_pins!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31);
