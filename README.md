# ws2812-flexspi
A neopixel driver based on NXP i.MX RT's FlexSPI bus.



# Examples

*- examples are intended for the [Teensy MicroMod](https://www.sparkfun.com/products/16402) module -*

## Prerequisites

The following hardware is required for the examples:
- A [Teensy MicroMod](https://www.sparkfun.com/products/16402) module
- A MicroMod carrier board, like the [SparkFun ATP Carrier Board](https://www.sparkfun.com/products/16885)
- A way to read the Teensy's UART, like a USB-UART-converter

The following software tools have to be installed:
- Python3 (as `python3`, or modify `run.py` to use the `python` binary)
- [`llvm-objcopy`](https://github.com/rust-lang/rust/issues/85658)
  - Install via `rustup component add llvm-tools-preview`
- [`teensy-loader-cli`](https://www.pjrc.com/teensy/loader_cli.html)


## Run

- Connect the Teensy to PC via USB cable.
- Run `cargo run --release --example rainbow`.
- Read the output of the examples on the Teensy's UART.
