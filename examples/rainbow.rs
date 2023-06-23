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
use common::{uart_log, UartWriter};

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

    let mut prepared_pixels1 = ws2812_flexio::PreparedPixels::<332, 3>::new();
    let mut prepared_pixels2 = ws2812_flexio::PreparedPixels::<332, 3>::new();

    let mut pixels = core::iter::from_fn({
        let mut pos = 0;
        move || {
            let p = pos as u32;
            pos = u32::from((pos + 1) as u8);
            Some([(3 * p) as u8, (3 * p + 1) as u8, (3 * p + 2) as u8])
        }
    });
    prepared_pixels1.prepare_pixels(&mut pixels);
    prepared_pixels2.prepare_pixels(&mut pixels);

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
    let mut neopixel = ws2812_flexio::flexio::Ws2812Driver::init(
        &mut ccm,
        flexio2,
        (pins.p6, pins.p7, pins.p8, pins.p9),
    )
    .unwrap();
    log::debug!("FlexIO initialized.");

    log::info!("Performing neopixel write ...");
    for _ in 0..10 {
        neopixel.write([Some(&prepared_pixels1), None, None, None]);
    }
    log::debug!("Write done.");

    // Blink with a cycle length of 2 seconds, to make it verifyable that
    // our timer runs at the correct speed.
    loop {
        let time_s = time_us() / 1_000_000;
        if time_s % 2 == 0 {
            led.set();
        } else {
            led.clear();
        }
    }
}
