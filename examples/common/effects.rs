use core::ops::Rem;

use palette::{Hsv, IntoColor, Srgb};

const DOT_DISTANCE: u32 = 25;

pub fn running_dots(t: u32, pixels: &mut [Srgb<f32>]) {
    for (pixel_pos, pixel_data) in pixels.iter_mut().enumerate() {
        let offset = t.wrapping_add(pixel_pos as u32);
        if (offset % DOT_DISTANCE) == 0 {
            *pixel_data = Srgb::new(0.0, 1.0, 0.0);
        } else {
            *pixel_data = Srgb::new(0.0, 0.0, 0.0);
        }
    }
}

pub fn rainbow(t: u32, pixels: &mut [Srgb<f32>]) {
    let t = (t as f32) / 500.;

    for (pixel_pos, pixel_data) in pixels.iter_mut().enumerate() {
        let offset = (t + (pixel_pos as f32) / 700.0).rem(1.0);

        let color = Hsv::new_srgb(360.0 * offset, 1.0, 1.0);
        *pixel_data = color.into_color();
    }
}

pub fn test_pattern(pixels: &mut [[u8; 3]]) {
    let mut val = 1;
    for pixel in pixels {
        for ch in pixel {
            *ch = val;
            val = val.wrapping_add(1);
        }
    }
}
