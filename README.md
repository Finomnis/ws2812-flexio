# ws2812-flexio
A neopixel driver based on NXP i.MX RT's FlexIO bus.



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
