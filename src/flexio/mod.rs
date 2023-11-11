use imxrt_ral as ral;

use ral::{flexio, Valid};

mod dma;
mod driver;
mod flexio_configurator;
mod idle_timer_finished_watcher;
mod interleaved_pixels;
mod interrupt_handler;
mod maybe_own;
mod preprocessed_pixels;

use crate::Pins;

pub use preprocessed_pixels::PreprocessedPixels;

use self::{idle_timer_finished_watcher::IdleTimerFinishedWatcher, maybe_own::MaybeOwn};

/// A WS2812 Neopixel LED Strip driver based on the i.MX RT FlexIO module
pub struct WS2812Driver<const N: u8, const L: usize, PINS: Pins<N, L>>
where
    flexio::Instance<N>: Valid,
{
    flexio: flexio::Instance<N>,
    _pins: PINS,
    idle_timer_finished: MaybeOwn<InterruptHandler<N>>,
}

/// The result of [WS2812Driver::write_dma()][WS2812Driver::write_dma].
pub struct WriteDmaResult<R> {
    /// The result of the concurrent function
    pub result: R,
    /// True if the concurrent function took longer than writing the
    /// data to the LED strips. This might indicate a render lag.
    pub lagged: bool,
}

/// TODO
pub struct InterruptHandler<const N: u8> {
    watcher: IdleTimerFinishedWatcher<N>,
}
