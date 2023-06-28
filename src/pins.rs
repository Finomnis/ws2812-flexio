use imxrt_hal as hal;

use hal::iomuxc::{self, flexio::Pin};

use paste::paste;

/// The pins to use for WS2812.
pub trait Pins<const N: u8, const L: usize> {
    /// The amount of pins this object contains.
    const PIN_COUNT: u8;

    /// Configures the pins.
    ///
    /// This is not intended to be called by the user;
    /// it will be used inside of the driver.
    fn configure(&mut self);

    /// The FlexIO pin offsets
    const FLEXIO_PIN_OFFSETS: &'static [u8];
}

macro_rules! count {
    () => (0u8);
    ( $x:tt $($xs:tt)* ) => (1u8 + count!($($xs)*));
}

macro_rules! impl_pins {
    ($($n:literal)+) => {
        paste! {
            impl<const N: u8, $([<P $n>]: Pin<N>),+> Pins<N, {count!($($n)+) as usize}> for ($([<P $n>]),+,) {
                fn configure(&mut self) {
                    $(
                        iomuxc::flexio::prepare(&mut self.$n);
                    )+
                }

                const PIN_COUNT: u8 = count!($($n)+);

                const FLEXIO_PIN_OFFSETS: &'static[u8] = &[$(
                    [<P $n>]::OFFSET
                ),+];
            }
        }
    };
}

impl_pins!(0);
impl_pins!(0 1);
impl_pins!(0 1 2);
impl_pins!(0 1 2 3);
