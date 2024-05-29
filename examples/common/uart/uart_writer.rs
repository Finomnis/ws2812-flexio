use core::fmt::Write;

struct Inner<W> {
    uart: W,
}

pub struct UartWriter<W> {
    inner: Inner<W>,
}

impl<W> UartWriter<W>
where
    W: embedded_io::Write,
{
    pub fn new(uart: W) -> Self {
        Self {
            inner: Inner { uart },
        }
    }

    pub fn write_fmt(&mut self, args: core::fmt::Arguments<'_>) {
        // Wrapper, to remove the necessity to unwrap().
        // Unwrap will always succeed, because imxrt's UARTs are `Infallible`.
        self.inner.write_fmt(args).unwrap();
    }
}

impl<W> core::fmt::Write for Inner<W>
where
    W: embedded_io::Write,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        assert!(s.is_ascii());
        for b in s.bytes() {
            match b {
                b'\r' => (),
                b'\n' => {
                    // Uart seems to usually be \r\n encoded.
                    // writeln!(), however, is always \n encoded,
                    // so convert here on the fly.
                    self.uart.write_all(b"\r\n").unwrap()
                }
                _ => self.uart.write_all(core::slice::from_ref(&b)).unwrap(),
            }
        }
        self.uart.flush().unwrap();
        Ok(())
    }
}
