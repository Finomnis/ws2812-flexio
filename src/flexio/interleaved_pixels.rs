use crate::pixelstream::PixelStreamRef;

fn spread4(x: u8) -> u32 {
    let mut x = u32::from(x);

    x = (x | (x << 12)) & 0x000F000F;
    x = (x | (x << 6)) & 0x03030303;
    x = (x | (x << 3)) & 0x11111111;

    x
}

pub struct InterleavedPixels<'a, const N: usize> {
    streams: [&'a mut dyn PixelStreamRef; N],
    leftover_trailing_bytes: u8,
}

impl<'a, const N: usize> InterleavedPixels<'a, N> {
    pub fn new(streams: [&'a mut dyn PixelStreamRef; N]) -> Self {
        Self {
            streams,
            leftover_trailing_bytes: 3,
        }
    }
}

impl<const N: usize> Iterator for InterleavedPixels<'_, N> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let mut has_next_data = false;
        let mut next_data = 0;
        for (pos, stream) in self.streams.iter_mut().take(4).enumerate() {
            if let Some(d) = stream.next() {
                next_data |= spread4(d) << (3 - pos);
                has_next_data = true;
            }
        }

        if has_next_data {
            Some(next_data)
        } else if self.leftover_trailing_bytes > 0 {
            self.leftover_trailing_bytes -= 1;
            Some(next_data)
        } else {
            None
        }
    }
}
