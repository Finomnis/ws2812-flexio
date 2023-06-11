// This example renders a moving rainbow on a 332 pixel long led strip using a Teensy MicroMod.
//
// Led Strip: https://www.ipixelleds.com/index.php?id=923
// Teensy Micromod: https://www.sparkfun.com/products/16402
//
// The Teensy Micromod is based on the imxrt-1062 MCU.

#![no_std]
#![no_main]

use teensy4_bsp as bsp;
use teensy4_panic as _;

use bsp::board;
use bsp::hal;

mod common;
use common::UartWriter;

#[bsp::rt::entry]
fn main() -> ! {
    let board::Resources {
        mut gpio2,
        pins,
        lpuart6,
        gpt1: mut us_timer,
        mut ccm,
        flexspi2,
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

    // Initialize timer
    // Is a 32-bit timer with us precision.
    // Overflows every 71.58 minutes, which is sufficient for our example.
    write!(uart, "Initializing timer ... ");
    assert_eq!(board::PERCLK_FREQUENCY, 1_000_000);
    us_timer.set_clock_source(hal::gpt::ClockSource::PeripheralClock);
    us_timer.set_divider(1);
    us_timer.set_mode(hal::gpt::Mode::FreeRunning);
    us_timer.enable();
    let time_us = move || us_timer.count();
    writeln!(uart, "done");

    // FlexSPI driver
    use ws2812_flexspi::flexspi::FlexSPI;
    write!(uart, "Initializing FlexSPI ... ");
    let _flexspi2 = FlexSPI::init(&mut ccm, flexspi2);
    writeln!(uart, "done");

    // Blink with a cycle length of 2 seconds, to make it verifyable that
    // our timer runs at the correct speed.
    loop {
        let time_s = time_us() / 1_000_000;
        if time_s % 2 == 0 {
            led.clear();
        } else {
            led.clear();
        }
    }
}
