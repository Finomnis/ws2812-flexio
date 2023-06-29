use crate::pixelstream::PixelStreamRef;

use super::interleaved_pixels::InterleavedPixels;

/// A buffer that preprocesses pixel data for FlexIO DMA usage.
///
/// # Generics:
///
/// * `N` - the number of pixels the buffer can hold
/// * `L` - the number of LED strips
/// * `P` - the number of bytes per pixel
#[derive(Debug)]
#[repr(C, align(4))]
pub struct PreprocessedPixels<const N: usize, const L: usize, const P: usize = 3> {
    /// Start with a `u32`, for 32bit alignment
    len: u32,
    /// The data. Would ideally be `[u32; P*N]`, but const expressions aren't there yet.
    /// So we need to trick it with a pointer reinterpret cast later.
    ///
    /// Note that this struct always stores data for four LED strips, even when less are used.
    /// The unused data is filled with zeros. That's just how the driver works.
    data: [[u32; P]; N],
    /// Zero termination of the data array.
    /// Be sure to use the same element size as `data`, because otherwise it might introduce padding.
    /// And I'm quite certain that reading from padding is undefined behaviour.
    zero_termination: [u32; P],
}

impl<const N: usize, const L: usize, const P: usize> PreprocessedPixels<N, L, P> {
    /// Creates a new PreprocessedPixels buffer.
    pub const fn new() -> Self {
        Self {
            len: 0,
            data: [[0; P]; N],
            zero_termination: [0; P],
        }
    }

    /// The amount of pixels that fit into this buffer
    pub fn capacity(&self) -> usize {
        N
    }

    fn get_data_mut(&mut self) -> &mut [u32] {
        let ptr = self.data.as_mut_ptr().cast();
        let len = P * N;

        /* SAFETY
            Our data is contiguous, so we can cast freely between [[u32;X];Y] and [u32;X*Y].
        */
        unsafe { core::slice::from_raw_parts_mut(ptr, len) }
    }

    pub(crate) fn get_dma_data(&self) -> &[u32] {
        let ptr = self.data.as_ptr().cast();

        let len = (N * P).min(self.len as usize) + P;

        /* SAFETY
            Our data is contiguous, so we can cast freely between [[u32;X];Y] and [u32;X*Y].
            The + P is also safe, because our `zero_termination` is directly after it, no padding bytes.
        */
        unsafe { core::slice::from_raw_parts(ptr, len) }
    }

    /// Prepares a set of pixels for transmission to the LED strip.
    pub fn prepare_pixels(&mut self, pixels: [&mut dyn PixelStreamRef; L]) {
        let data = self.get_data_mut();

        let mut len = 0;
        for (d, pixel) in data.iter_mut().zip(InterleavedPixels::new(pixels)) {
            *d = pixel;
            len += 1;
        }

        data[len..].fill(0);

        self.len = len as u32;
    }
}

impl<const N: usize, const P: usize> Default for PreprocessedPixels<N, P> {
    fn default() -> Self {
        Self::new()
    }
}
