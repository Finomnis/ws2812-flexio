// This example renders a moving rainbow on a 332 pixel long led strip using a Teensy MicroMod.
//
// Led Strip: https://www.ipixelleds.com/index.php?id=923
// Teensy Micromod: https://www.sparkfun.com/products/16402
//
// The Teensy Micromod is based on the imxrt-1062 MCU.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)] // Required for RTIC 2.0 right now

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

use ws2812_flexio::{IntoPixelStream, PreprocessedPixels, WS2812Driver};

const NUM_PIXELS: usize = 332;

fn linearize_color(col: &Srgb) -> LinSrgb<u8> {
    col.into_linear().into_format()
}

fn render_frame(
    t: u32,
    framebuffer_0: &mut [Srgb],
    framebuffer_1: &mut [Srgb],
    framebuffer_2: &mut [[u8; 3]],
    render_buffer: &mut PreprocessedPixels<NUM_PIXELS, 3>,
) {
    effects::running_dots(t, framebuffer_0);
    effects::rainbow(t, framebuffer_1);
    effects::test_pattern(framebuffer_2);

    render_buffer.prepare_pixels([
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
}

#[rtic::app(device = teensy4_bsp, dispatchers = [LPSPI1])]
mod app {
    use bsp::pins::common::{P6, P7, P8};

    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: board::Led,
        us_timer: hal::gpt::Gpt1,
        neopixel: WS2812Driver<2, 3, (P6, P7, P8)>,
        neopixel_dma: hal::dma::channel::Channel,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let board::Resources {
            mut gpio2,
            pins,
            lpuart6,
            gpt1: mut us_timer,
            ccm,
            flexio2,
            mut dma,
            ..
        } = board::t40(cx.device);

        // Initialize LED
        let led = board::led(&mut gpio2, pins.p13);
        led.set();

        // Initialize UART
        let mut uart = UartWriter::new(board::lpuart(lpuart6, pins.p1, pins.p0, 115200));
        writeln!(uart);

        // Write welcome message
        writeln!(uart, "===== WS2812 Rainbow Example (with DMA!) =====");
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
        log::debug!("Timer initialized.");

        // Initialize DMA
        let mut neopixel_dma = dma[0].take().unwrap();
        neopixel_dma.set_interrupt_on_completion(true);
        unsafe {
            cortex_m::peripheral::NVIC::unmask(imxrt_ral::interrupt::DMA0_DMA16);
        }

        // Ws2812 driver
        log::info!("Initializing FlexIO ...");
        // Set FlexIO clock to 16Mhz, as required by the driver
        ral::modify_reg!(
            ral::ccm,
            ccm,
            CS1CDR,
            FLEXIO2_CLK_PRED: FLEXIO2_CLK_PRED_4,
            FLEXIO2_CLK_PODF: DIVIDE_6,
        );
        let neopixel = WS2812Driver::init(flexio2, (pins.p6, pins.p7, pins.p8)).unwrap();
        log::debug!("FlexIO initialized.");

        // Spawn animation task
        animate::spawn().unwrap();

        (
            Shared {},
            Local {
                led,
                us_timer,
                neopixel,
                neopixel_dma,
            },
        )
    }

    #[task(binds = DMA0_DMA16)]
    fn on_dma(_cx: on_dma::Context) {
        unsafe {
            imxrt_hal::dma::DMA.on_interrupt(0);
            imxrt_hal::dma::DMA.on_interrupt(16);
        }
    }

    #[task(local = [
        led, us_timer, neopixel, neopixel_dma,
        framebuffer_0: [Srgb; NUM_PIXELS] = [Srgb::new(0., 0., 0.); NUM_PIXELS],
        framebuffer_1: [Srgb; NUM_PIXELS] = [Srgb::new(0., 0., 0.); NUM_PIXELS],
        framebuffer_2: [[u8; 3]; NUM_PIXELS] = [[0; 3]; NUM_PIXELS],
        buffers: (
            PreprocessedPixels<NUM_PIXELS, 3>,
            PreprocessedPixels<NUM_PIXELS, 3>,
        ) = (PreprocessedPixels::new(), PreprocessedPixels::new())
    ])]
    async fn animate(cx: animate::Context) {
        let animate::LocalResources {
            led,
            us_timer,
            framebuffer_0,
            framebuffer_1,
            framebuffer_2,
            buffers,
            neopixel,
            neopixel_dma,
            ..
        } = cx.local;

        let time_us = || us_timer.count();

        let mut flip_buffers = false;

        let mut t = 0;

        let mut t_last = time_us() as i32;

        loop {
            let (render_buffer, display_buffer) = if flip_buffers {
                (&mut buffers.0, &buffers.1)
            } else {
                (&mut buffers.1, &buffers.0)
            };
            flip_buffers = !flip_buffers;

            let lagged = neopixel
                .write_dma(display_buffer, neopixel_dma, 1, async {
                    t += 1;

                    render_frame(
                        t,
                        framebuffer_0,
                        framebuffer_1,
                        framebuffer_2,
                        render_buffer,
                    );

                    led.toggle();

                    if (t % 100) == 0 {
                        let t_now = time_us() as i32;
                        let t_diff = (t_now).wrapping_sub(t_last);
                        t_last = t_now;

                        let t_diff = (t_diff as f32) / 1_000_000.0;
                        let fps = 100.0 / t_diff;

                        log::info!("Frames: {}, FPS: {:.02}", t, fps);
                    }
                })
                .await
                .unwrap()
                .lagged;

            if lagged {
                // Note that with the current implementation of this
                // example, it is expected that the first frame lags.
                // Reason is that we use double buffering, and while
                // rendering the first frame, the other buffer will get
                // displayed, which is empty. And displaying an empty
                // buffer is really fast.
                //
                // This could be fixed by pre-rendering the first frame,
                // but was left in here intentionally to demonstrate
                // this feature.
                log::warn!("Frame {} lagged.", t - 1);
            }
        }
    }
}
