use super::InterruptHandler;

impl<const N: u8> InterruptHandler<N> {
    /// Signals to the [`WS2812Driver`](crate::WS2812Driver) that a FlexIO
    /// interrupt happened.
    ///
    /// Needs to be called inside of the respective FlexIO
    /// interrupt handler function.
    ///
    /// See the [examples](https://github.com/Finomnis/ws2812-flexio/tree/main/examples) for more information.
    pub fn on_interrupt(&self) {
        self.data.finished_watcher.on_interrupt();
    }
}
