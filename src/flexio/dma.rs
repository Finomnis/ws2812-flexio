use imxrt_hal::dma;
use imxrt_ral::{flexio, write_reg};

pub(crate) struct WS2812Dma<'a, const N: u8> {
    flexio: &'a mut flexio::Instance<N>,
    shifter_id: u8,
    dma_channel: u32,
}

impl<'a, const N: u8> WS2812Dma<'a, N> {
    pub fn new(flexio: &'a mut flexio::Instance<N>, shifter_id: u8, dma_channel: u32) -> Self {
        Self {
            flexio,
            shifter_id,
            dma_channel,
        }
    }
}

unsafe impl<const N: u8> dma::peripheral::Destination<u32> for WS2812Dma<'_, N> {
    fn destination_signal(&self) -> u32 {
        self.dma_channel
    }

    fn destination_address(&self) -> *const u32 {
        let buf = &self.flexio.SHIFTBUFBIS[usize::from(self.shifter_id)];

        let buf_ptr: *const _ = buf;
        buf_ptr.cast()
    }

    fn enable_destination(&mut self) {
        let dma_reg = 1 << self.shifter_id;
        write_reg!(flexio, self.flexio, SHIFTSDEN, dma_reg);
    }

    fn disable_destination(&mut self) {
        let dma_reg = 0;
        write_reg!(flexio, self.flexio, SHIFTSDEN, dma_reg);
    }
}
