use super::Pixel;

/// A buffer that prepares pixel data for FlexIO usage.
#[repr(C, align(4))]
pub struct PreparedPixels<const N: usize, const P: usize> {
    // Start with a `u32`, for 32bit alignment
    len: u32,
    // The data. Would ideally be `[u32; P*N/4]`, but const expressions aren't there yet.
    // So we need to trick it with a pointer reinterpret cast later.
    data: [[u8; P]; N],
    // Zero termination of the data array.
    // Be sure to use `[u8]` here, because a `u32` might introduce a padding.
    // And I'm quite certain that reading from padding is undefined behaviour.
    zero_termination: [u8; 4],
}

impl<const N: usize, const P: usize> PreparedPixels<N, P> {
    /// Creates a new PreparedPixels buffer.
    pub fn new() -> Self {
        Self {
            len: 0,
            data: [[0; P]; N],
            zero_termination: [0; 4],
        }
    }

    /// The amount of pixels that fit into this buffer
    pub fn capacity(&self) -> usize {
        N
    }

    /// Prepares a set of pixels for transmission to the LED strip.
    pub fn prepare_pixels<T: Pixel<P>>(&mut self, pixels: impl Iterator<Item = T>) {
        let mut len: usize = 0;
        for (d, pixel) in self.data.iter_mut().zip(pixels) {
            *d = pixel.get_ws2812_bytes();
            len += 1;
        }

        self.data[len..].fill([0; P]);
        self.len = len as u32;
    }
}

impl<const N: usize, const P: usize> Default for PreparedPixels<N, P> {
    fn default() -> Self {
        Self::new()
    }
}

/// A reference to a PreparedPixels buffer.
///
/// Used as an abstraction to pass prepared pixels
/// of different sizes to the FlexIO driver.
pub trait PreparedPixelsRef {
    /// Retrieves a reference to the DMA data buffer.
    ///
    /// Don't use this in user code; it is meant for internal use.
    fn get_dma_buffer(&self) -> &[u32];
}

impl<const N: usize, const P: usize> PreparedPixelsRef for PreparedPixels<N, P> {
    /// Returns the DMA buffer of pixel data.
    ///
    /// Terminates with at least one byte of zeros,
    /// which is required by the FlexIO code.
    fn get_dma_buffer(&self) -> &[u32] {
        let len = N.min(self.len as usize);

        let ptr = self.data.as_ptr().cast();
        let len = P * len;

        let len_32 = (len + 4) / 4;

        /* SAFETY
            We can assume that `self.data` is 32bit aligned.

            Then, we need exactly so many bytes zero termination
            that our `u32` slice contains at least 1 and at most
            4 bytes zero padding.

            len 0 => len32 1 => 4 bytes zero-padding
            len 1 => len32 1 => 3 bytes zero-padding
            len 2 => len32 1 => 2 bytes zero-padding
            len 3 => len32 1 => 1 byte  zero-padding
            len 4 => len32 2 => 4 bytes zero-padding
            len 5 => len32 2 => 3 bytes zero-padding
            len 6 => len32 2 => 2 bytes zero-padding
            len 7 => len32 2 => 1 byte  zero-padding
            len 8 => len32 3 => 4 bytes zero-padding

            This means our formula is `(len+4)/4`.
        */
        unsafe { core::slice::from_raw_parts(ptr, len_32) }
    }
}
