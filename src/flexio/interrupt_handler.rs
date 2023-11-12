use super::InterruptHandler;

impl<const N: u8> InterruptHandler<N> {
    /// TODO
    pub fn on_interrupt(&self) {
        self.data.watcher.on_interrupt();
    }
}
