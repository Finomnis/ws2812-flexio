use imxrt_hal as hal;
use imxrt_ral as ral;

use hal::ccm::clock_gate;
use ral::{flexio, Valid};

/// Errors the driver can cause
pub mod errors;
mod pins;

pub use pins::Pins;

/// A WS2812 Neopixel LED Strip driver based on the i.MX RT FlexIO module
pub struct Ws2812Driver<const N: u8, PINS: Pins<N>>
where
    flexio::Instance<N>: Valid,
{
    flexio: flexio::Instance<N>,
    _pins: PINS,
}

impl<const N: u8, PINS: Pins<N>> Ws2812Driver<N, PINS>
where
    flexio::Instance<N>: Valid,
{
    /// Initializes the FlexIO driver.
    ///
    /// IMPORTANT! Make sure that the clock input of the FlexIO instance is at 60MHz
    /// prior to calling this function.
    pub fn init(
        ccm: &mut ral::ccm::CCM,
        flexio: flexio::Instance<N>,
        mut pins: PINS,
    ) -> Result<Self, errors::WS2812InitError> {
        // Configure clocks
        clock_gate::flexio::<N>().set(ccm, clock_gate::ON);

        // Parameter check
        let (version_major, version_minor, available_feature_set) =
            ral::read_reg!(ral::flexio, flexio, VERID, MAJOR, MINOR, FEATURE);
        let (available_triggers, available_pins, available_timers, available_shifters) =
            ral::read_reg!(ral::flexio, flexio, PARAM, TRIGGER, PIN, TIMER, SHIFTER);

        log::debug!("Initializing FlexIO #{}.", N);
        log::debug!("    Version: {}.{}", version_major, version_minor);
        log::debug!("    Feature Set: {}", available_feature_set);
        log::debug!("    Peripherals:");
        log::debug!("        {} triggers", available_triggers);
        log::debug!("        {} pins", available_pins);
        log::debug!("        {} timers", available_timers);
        log::debug!("        {} shifters", available_shifters);
        log::debug!("Pin Offsets: {:?}", PINS::FLEXIO_PIN_OFFSETS);

        if available_pins < PINS::PIN_COUNT {
            return Err(errors::WS2812InitError::NotEnoughPins);
        }

        // if available_shifters < PINS::PIN_COUNT {
        //     return Err(errors::WS2812InitError::NotEnoughShifters);
        // }

        // TODO check timers, shifters and triggers count

        // Configure pins
        pins.configure();

        //////////// Configure FlexIO registers /////////////////

        // Reset
        ral::write_reg!(ral::flexio, flexio, CTRL, SWRST: 1);
        assert!(ral::read_reg!(ral::flexio, flexio, CTRL, SWRST == 1));
        ral::write_reg!(ral::flexio, flexio, CTRL, SWRST: 0);
        while ral::read_reg!(ral::flexio, flexio, CTRL, SWRST == 1) {}

        // Configure Shifters
        ral::write_reg!(ral::flexio, flexio, SHIFTCTL[0],
            TIMSEL: 0, // Use timer 0 for our clock input
            TIMPOL: 0, // Shift on positive edge of the timer
            PINCFG: 0b11, // Output to a pin. TODO: only output to high-bit timer
            PINSEL: u32::from(PINS::FLEXIO_PIN_OFFSETS[0]), // Output pin
            PINPOL: 1, // Pin polarity
            SMOD: 0b010, // Transmit mode
        );
        ral::write_reg!(ral::flexio, flexio, SHIFTCFG[0],
            PWIDTH: 0, // Single bit shift-width
            INSRC: 0, // Input source; irrelevant for transmit mode
            SSTOP: 0b00, // No stop bit
            SSTART: 0b01, // No start bit, load data on first shift
        );

        // for (pos, pin) in PINS::FLEXIO_PIN_OFFSETS.iter().copied().enumerate() {
        //     //ral::write_reg!(ral::flexio, flexio,)
        // }

        // Timer0
        ral::write_reg!(ral::flexio, flexio, TIMCMP[0], ((32 * 2 - 1) << 8) | 74);
        ral::write_reg!(ral::flexio, flexio, TIMCTL[0],
            TRGSEL: 0b0001, // Use shifter 0 flag as trigger
            TRGPOL: 1, // Trigger whan shifter0 got filled
            TRGSRC: 1, // Internal trigger
            PINSEL: u32::from(PINS::FLEXIO_PIN_OFFSETS[1]),
            PINCFG: 0b11, // Pin output enabled
            PINPOL: 0, // Active high
            TIMOD: 0b01, // 8-bit dual counter baud/bit mode
        );
        ral::write_reg!(ral::flexio, flexio, TIMCFG[0],
            TIMOUT: 0b01, // Zero when enabled, not affected by reset
            TIMDEC: 0b00, // Input clock from FlexIO clock
            TIMRST: 0b110, // Never reset
            TIMDIS: 0b010, // Disabled on timer compare (upper 8 bits match and decrement)
            TIMENA: 0b010, // Enabled on trigger high
            TSTOP: 0b00, // No stop bit
            TSTART: 0b00, // No start bit
        );

        let pin_mask = PINS::FLEXIO_PIN_OFFSETS
            .into_iter()
            .map(|v| 1 << v)
            .fold(0, |a, b| a | b);
        log::debug!("Mask: {:#010x}", pin_mask);

        // Enable
        ral::write_reg!(ral::flexio, flexio, CTRL, FLEXEN: 1);
        let reg = ral::read_reg!(ral::flexio, flexio, CTRL);
        log::debug!("Ctrl: {:#010x}", reg);

        Ok(Self {
            flexio,
            _pins: pins,
        })
    }

    /// A dummy function for development purposes
    pub fn dummy_write(&mut self) {
        ral::write_reg!(ral::flexio, self.flexio, SHIFTBUF[0], 0x555500ff);
    }
}

// TODO: Add PreparedPixels struct that contains prepared pixels.
// TODO: Add Iterator<Item = (Pixel, Pixel)> based pixel-setter function.
