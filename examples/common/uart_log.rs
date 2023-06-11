use core::{
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering},
};

use critical_section::Mutex;
use log::{LevelFilter, Log};
use teensy4_bsp::board::Lpuart6;

use super::UartWriter;

static LOG_INITIALIZED: AtomicBool = AtomicBool::new(false);
static mut LOGGER: Option<UartLogger> = None;

pub fn init(uart: UartWriter<Lpuart6>, max_level: LevelFilter) {
    let already_initialized = LOG_INITIALIZED.swap(true, Ordering::SeqCst);

    if !already_initialized {
        let logger = unsafe { LOGGER.insert(UartLogger::new(uart)) };

        if let Err(e) = log::set_logger(logger) {
            critical_section::with(|cs| {
                writeln!(
                    logger.uart.borrow(cs).borrow_mut(),
                    "Unable to set logger: {}",
                    e
                );
            });
        }

        log::set_max_level(max_level);
    }
}

struct UartLogger {
    uart: Mutex<RefCell<UartWriter<Lpuart6>>>,
}

impl UartLogger {
    fn new(uart: UartWriter<Lpuart6>) -> Self {
        Self {
            uart: Mutex::new(RefCell::new(uart)),
        }
    }
}

impl Log for UartLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        critical_section::with(|cs| {
            let mut uart = self.uart.borrow(cs).borrow_mut();
            writeln!(
                uart,
                "{}:{} -- {}",
                record.level(),
                record.target(),
                record.args()
            );
        })
    }

    fn flush(&self) {
        // Nothing, UartLogger already flushes
    }
}
