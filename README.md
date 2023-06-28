# ws2812-flexio
A neopixel driver based on NXP i.MX RT's FlexIO bus.

## Pixel Type

The default color space of NeoPixel LED strips is 8-bit [linear sRGB](https://matt77hias.github.io/blog/2018/07/01/linear-gamma-and-sRGB-color-spaces.html), therefore the recommended pixel type is [`LinSrgb<u8>`](https://docs.rs/palette/latest/palette/type.LinSrgb.html).

Be aware that this differs from normal, gamma corrected sRGB; a conversion has to take place.
More info can be found in the documentation of the [`palette`](https://docs.rs/palette) crate.

## Specs

- Parallel strips:

  The library can drive multiple strips in parallel. To be more specific, driving multiple strips from the same FlexIO instance requires `2 + 2 * strips` FlexIO timers, so a for example a FlexIO instance with `8` timers can drive `3` strips in parallel.

- Framerate:

  The library drives the LED strips at 800kHz with a latch time of 300us. This gives us the following formula:

  $$
  fps = {100.000 \over {num\_pixels \times bytes\_per\_pixel + 30}}
  $$

  Example: For a strip with [332 SK6805 pixels](https://www.ipixelleds.com/index.php?id=923), we can achieve $100.000 / (332*3 + 30) = 97.5$ fps.

  Be aware that this framerate is only realistic for the DMA based driver; at the blocking driver, additional time gets lost while the next frame gets computed.


# Examples

*- examples are intended for the [Teensy 4.0](https://www.pjrc.com/store/teensy40.html) board -*

## Prerequisites

The following hardware is required for the examples:
- A [Teensy 4.0](https://www.pjrc.com/store/teensy40.html) development board
- A way to read the Teensy's UART, like a USB-UART-converter

The following software tools have to be installed:
- Python3 (as `python3`, or modify `run.py` to use the `python` binary)
- `llvm-objcopy`
  - Install [`LLVM`](https://github.com/llvm/llvm-project/releases) tool suite
- [`teensy-loader-cli`](https://www.pjrc.com/teensy/loader_cli.html)


## Run

- Connect the Teensy to PC via USB cable.
- Run `cargo run --release --example triple_332`.
- Read the output of the examples on the Teensy's UART.
- Pin 6, 7 and 8 output data for NeoPixel RGB strips of length 332 each
  (for example [P/N: S010332ZA3SA8](https://www.ipixelleds.com/index.php?id=923)).
  Note that those pins are at 3.3V, and most NeoPixel LED strips require a 5V data signal, which means an external level shifter is required.
