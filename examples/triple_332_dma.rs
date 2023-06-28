// This example renders a moving rainbow on a 332 pixel long led strip using a Teensy MicroMod.
//
// Led Strip: https://www.ipixelleds.com/index.php?id=923
// Teensy Micromod: https://www.sparkfun.com/products/16402
//
// The Teensy Micromod is based on the imxrt-1062 MCU.

#![no_std]
#![no_main]

use teensy4_bsp as bsp;

use bsp::board;
use bsp::hal;
use bsp::ral;

mod common;
use common::{
    effects,
    uart::{uart_log, UartWriter},
};

use palette::LinSrgb;
use palette::Srgb;

const NUM_PIXELS: usize = 332;

fn linearize_color(col: &Srgb) -> LinSrgb<u8> {
    col.into_linear().into_format()
}

#[bsp::rt::entry]
fn main() -> ! {
    let board::Resources {
        mut gpio2,
        pins,
        lpuart6,
        gpt1: mut us_timer,
        mut ccm,
        flexio2,
        ..
    } = board::t40(board::instances());

    // Initialize LED
    let led = board::led(&mut gpio2, pins.p13);
    led.set();

    // Initialize UART
    let mut uart = UartWriter::new(board::lpuart(lpuart6, pins.p1, pins.p0, 115200));
    writeln!(uart);

    // Write welcome message
    writeln!(uart, "===== WS2812 Rainbow Example =====");
    writeln!(uart);

    // Initialize logging
    uart_log::init(uart, log::LevelFilter::Debug);

    // Initialize timer
    // Is a 32-bit timer with us precision.
    // Overflows every 71.58 minutes, which is sufficient for our example.
    log::info!("Initializing timer ...");
    assert_eq!(board::PERCLK_FREQUENCY, 1_000_000);
    us_timer.set_clock_source(hal::gpt::ClockSource::PeripheralClock);
    us_timer.set_divider(1);
    us_timer.set_mode(hal::gpt::Mode::FreeRunning);
    us_timer.enable();
    let time_us = move || us_timer.count();
    log::debug!("Timer initialized.");

    // Ws2812 driver
    log::info!("Initializing FlexIO ...");
    // Set FlexIO clock to 16Mhz, as required by the driver
    ral::modify_reg!(
        ral::ccm,
        ccm,
        CS1CDR,
        FLEXIO1_CLK_PRED: FLEXIO1_CLK_PRED_4,
        FLEXIO1_CLK_PODF: DIVIDE_6,
    );
    let mut neopixel =
        ws2812_flexio::WS2812Driver::init(&mut ccm, flexio2, (pins.p6, pins.p7, pins.p8)).unwrap();
    log::debug!("FlexIO initialized.");

    let mut framebuffer_0 = [Srgb::new(0., 0., 0.); NUM_PIXELS];
    let mut framebuffer_1 = [Srgb::new(0., 0., 0.); NUM_PIXELS];
    let mut framebuffer_2 = [[0; 3]; NUM_PIXELS];

    let mut t = 0;

    let mut t_last = time_us() as i32;

    loop {
        use ws2812_flexio::IntoPixelStream;

        effects::running_dots(t, &mut framebuffer_0);
        effects::rainbow(t, &mut framebuffer_1);
        effects::test_pattern(&mut framebuffer_2);

        t += 1;

        neopixel.write([
            &mut framebuffer_0
                .iter()
                .map(linearize_color)
                .into_pixel_stream(),
            &mut framebuffer_1
                .iter()
                .map(linearize_color)
                .into_pixel_stream(),
            &mut framebuffer_2.into_pixel_stream(),
        ]);

        led.toggle();

        if (t % 100) == 0 {
            let t_now = time_us() as i32;
            let t_diff = (t_now).wrapping_sub(t_last);
            t_last = t_now;

            let t_diff = (t_diff as f32) / 1_000_000.0;
            let fps = 100.0 / t_diff;

            log::info!("Frames: {}, FPS: {:.02}", t, fps);
        }
    }
}
