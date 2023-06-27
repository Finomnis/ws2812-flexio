use core::cell::RefCell;

use imxrt_hal::dma;
use imxrt_ral::{flexio, read_reg, write_reg};

pub struct WS2812Dma<'a, const N: u8> {
    flexio: RefCell<&'a mut flexio::Instance<N>>,
    shifter_id: u8,
    dma_channel: u32,
}

impl<'a, const N: u8> WS2812Dma<'a, N> {
    pub fn new(
        flexio: RefCell<&'a mut flexio::Instance<N>>,
        shifter_id: u8,
        dma_channel: u32,
    ) -> Self {
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
        #[cfg(target_endian = "big")]
        let buf = &self.flexio.borrow().SHIFTBUFBIS[usize::from(self.shifter_id)];

        #[cfg(target_endian = "little")]
        let buf = &self.flexio.borrow().SHIFTBUFBBS[usize::from(self.shifter_id)];

        let buf_ptr: *const _ = buf;
        buf_ptr.cast()
    }

    fn enable_destination(&mut self) {
        let flexio = self.flexio.borrow_mut();
        let mut dma_reg = read_reg!(flexio, flexio, SHIFTSDEN);
        dma_reg |= 1 << self.shifter_id;
        write_reg!(flexio, flexio, SHIFTSDEN, dma_reg);
    }

    fn disable_destination(&mut self) {
        let flexio = self.flexio.borrow_mut();
        let mut dma_reg = read_reg!(flexio, flexio, SHIFTSDEN);
        dma_reg &= !(1 << self.shifter_id);
        write_reg!(flexio, flexio, SHIFTSDEN, dma_reg);
    }
}
