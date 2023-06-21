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

// A total cycle is 20 clock cycles. (16 MHz / 20 = 800 kHz)
const CLOCK_DIVIDER: u8 = 10; // Timer toggles; meaning we need two cycles for one timer clock cycle, so this is half the total cycle length
const LOW_BIT_CYCLES_ON: u8 = 5;
const HIGH_BIT_CYCLES_ON: u8 = 15;

// Newest WS2812 Version requires 300us latch time
const LATCH_DELAY_PIXELS: u16 = 240;

const CYCLE_LENGTH: u8 = CLOCK_DIVIDER * 2;
const LOW_BIT_CYCLES_OFF: u8 = CYCLE_LENGTH - LOW_BIT_CYCLES_ON;
const HIGH_BIT_CYCLES_OFF: u8 = CYCLE_LENGTH - HIGH_BIT_CYCLES_ON;
const LATCH_DELAY: u16 = CYCLE_LENGTH as u16 * LATCH_DELAY_PIXELS;

struct DriverBuilder<const N: u8, PINS: Pins<N>>
where
    flexio::Instance<N>: Valid,
{
    flexio: flexio::Instance<N>,
    pins: PINS,
}

impl<const N: u8, PINS: Pins<N>> DriverBuilder<N, PINS>
where
    flexio::Instance<N>: Valid,
{
    pub fn new(flexio: flexio::Instance<N>, mut pins: PINS) -> Self {
        // Configure pins
        pins.configure();

        // Reset
        ral::write_reg!(ral::flexio, flexio, CTRL, SWRST: SWRST_1);
        assert!(ral::read_reg!(ral::flexio, flexio, CTRL, SWRST == SWRST_1));
        ral::write_reg!(ral::flexio, flexio, CTRL, SWRST: SWRST_0);
        while ral::read_reg!(ral::flexio, flexio, CTRL, SWRST == SWRST_1) {}

        Self { flexio, pins }
    }

    pub fn build(self) -> Ws2812Driver<N, PINS> {
        // Enable
        ral::write_reg!(ral::flexio, self.flexio, CTRL, FLEXEN: FLEXEN_1);

        Ws2812Driver {
            flexio: self.flexio,
            _pins: self.pins,
        }
    }

    pub fn configure_shifter(&mut self, shifter_id: u8, input_timer: u8, output_pin: u8) {
        ral::write_reg!(ral::flexio, self.flexio, SHIFTCTL[usize::from(shifter_id)],
            TIMSEL: u32::from(input_timer), // Use timer 0 for our clock input
            TIMPOL: TIMPOL_0, // Shift on positive edge of the timer
            PINCFG: PINCFG_3, // Output to a pin. TODO: only output to high-bit timer
            PINSEL: u32::from(output_pin), // Output pin
            PINPOL: PINPOL_0, // Pin polarity
            SMOD: SMOD_2, // Transmit mode
        );
        ral::write_reg!(ral::flexio, self.flexio, SHIFTCFG[usize::from(shifter_id)],
            PWIDTH: 0, // Single bit shift-width
            INSRC: INSRC_0, // Input source; irrelevant for transmit mode
            SSTOP: SSTOP_0, // No stop bit
            SSTART: SSTART_1, // No start bit, load data on first shift
        );
    }

    pub fn configure_shift_timer(&mut self, timer_id: u8, shifter_id: u8, output_pin: u8) {
        const CYCLES_PER_SHIFTBUFFER: u32 = 32 * 2;

        ral::write_reg!(
            ral::flexio,
            self.flexio,
            TIMCMP[usize::from(timer_id)],
            ((CYCLES_PER_SHIFTBUFFER - 1) << 8) | (u32::from(CLOCK_DIVIDER) - 1)
        );
        ral::write_reg!(ral::flexio, self.flexio, TIMCTL[usize::from(timer_id)],
            TRGSEL: u32::from(shifter_id) * 4 + 1, // Use shifter flag as trigger
            TRGPOL: TRGPOL_1, // Trigger when shifter got filled
            TRGSRC: TRGSRC_1, // Internal trigger
            PINSEL: u32::from(output_pin),
            PINCFG: PINCFG_3, // Pin output enabled
            PINPOL: PINPOL_0, // Active high
            TIMOD: TIMOD_1, // 8-bit dual counter baud/bit mode
        );
        ral::write_reg!(ral::flexio, self.flexio, TIMCFG[usize::from(timer_id)],
            TIMOUT: TIMOUT_1, // Zero when enabled, not affected by reset
            TIMDEC: TIMDEC_0, // Input clock from FlexIO clock
            TIMRST: TIMRST_0, // Never reset
            TIMDIS: TIMDIS_2, // Disabled on timer compare (upper 8 bits match and decrement)
            TIMENA: TIMENA_2, // Enabled on trigger high
            TSTOP: TSTOP_0, // No stop bit
            TSTART: TSTART_0, // No start bit
        );
    }

    pub fn configure_low_bit_timer(&mut self, timer_id: u8, shift_timer_pin: u8, output_pin: u8) {
        ral::write_reg!(
            ral::flexio,
            self.flexio,
            TIMCMP[usize::from(timer_id)],
            (u32::from(LOW_BIT_CYCLES_OFF - 1) << 8) | (u32::from(LOW_BIT_CYCLES_ON - 1))
        );
        ral::write_reg!(ral::flexio, self.flexio, TIMCTL[usize::from(timer_id)],
            TRGSEL: u32::from(shift_timer_pin) * 2, // Use shift timer output as trigger
            TRGPOL: TRGPOL_0, // Trigger when shift timer output gets high
            TRGSRC: TRGSRC_1, // Internal trigger
            PINSEL: u32::from(output_pin),
            PINCFG: PINCFG_3, // Pin output enabled
            PINPOL: PINPOL_0, // Active high
            TIMOD: TIMOD_2, // 8-bit PWM mode
        );
        ral::write_reg!(ral::flexio, self.flexio, TIMCFG[usize::from(timer_id)],
            TIMOUT: TIMOUT_0, // One when enabled, not affected by reset
            TIMDEC: TIMDEC_0, // Input clock from FlexIO clock
            TIMRST: TIMRST_0, // Never reset
            TIMDIS: TIMDIS_2, // Disabled on timer compare (upper 8 bits match and decrement)
            TIMENA: TIMENA_6, // Enabled on trigger rising edge
            TSTOP: TSTOP_0, // No stop bit
            TSTART: TSTART_0, // No start bit
        );
    }

    pub fn configure_high_bit_timer(&mut self, timer_id: u8, shift_pin: u8, output_pin: u8) {
        ral::write_reg!(
            ral::flexio,
            self.flexio,
            TIMCMP[usize::from(timer_id)],
            (u32::from(HIGH_BIT_CYCLES_OFF - 1) << 8) | (u32::from(HIGH_BIT_CYCLES_ON - 1))
        );
        ral::write_reg!(ral::flexio, self.flexio, TIMCTL[usize::from(timer_id)],
            TRGSEL: u32::from(shift_pin) * 2, // Use shift output as trigger
            TRGPOL: TRGPOL_0, // Trigger when shift output gets high
            TRGSRC: TRGSRC_1, // Internal trigger
            PINSEL: u32::from(output_pin),
            PINCFG: PINCFG_3, // Pin output enabled
            PINPOL: PINPOL_0, // Active high
            TIMOD: TIMOD_2, // 8-bit PWM mode
        );
        ral::write_reg!(ral::flexio, self.flexio, TIMCFG[usize::from(timer_id)],
            TIMOUT: TIMOUT_0, // One when enabled, not affected by reset
            TIMDEC: TIMDEC_0, // Input clock from FlexIO clock
            TIMRST: TIMRST_0, // Never reset
            TIMDIS: TIMDIS_6, // Disabled on trigger falling edge
            TIMENA: TIMENA_6, // Enabled on trigger rising edge
            TSTOP: TSTOP_0, // No stop bit
            TSTART: TSTART_0, // No start bit
        );
    }

    pub fn configure_idle_timer(
        &mut self,
        timer_id: u8,
        shift_timer_pin: u8,
        output_pin: Option<u8>,
    ) {
        ral::write_reg!(
            ral::flexio,
            self.flexio,
            TIMCMP[usize::from(timer_id)],
            u32::from(LATCH_DELAY)
        );
        ral::write_reg!(ral::flexio, self.flexio, TIMCTL[usize::from(timer_id)],
            TRGSEL: u32::from(shift_timer_pin) * 2, // Use shift output as trigger
            TRGPOL: TRGPOL_0, // Trigger when shift output gets high
            TRGSRC: TRGSRC_1, // Internal trigger
            PINSEL: u32::from(output_pin.unwrap_or_default()),
            PINCFG: if output_pin.is_some() {PINCFG_3} else {PINCFG_0}, // Pin output enabled
            PINPOL: PINPOL_0, // Active high
            TIMOD: TIMOD_3, // 8-bit PWM mode
        );
        ral::write_reg!(ral::flexio, self.flexio, TIMCFG[usize::from(timer_id)],
            TIMOUT: TIMOUT_2,
            TIMDEC: TIMDEC_0, // Input clock from FlexIO clock
            TIMRST: TIMRST_6, // Reset on trigger rising edge
            TIMDIS: TIMDIS_2, // Disable on timer over
            TIMENA: TIMENA_6, // Enabled on trigger rising edge
            TSTOP: TSTOP_0, // No stop bit
            TSTART: TSTART_0, // No start bit
        );
    }
}

impl<const N: u8, PINS: Pins<N>> Ws2812Driver<N, PINS>
where
    flexio::Instance<N>: Valid,
{
    /// Initializes the FlexIO driver.
    ///
    /// IMPORTANT! Make sure that the clock input of the FlexIO instance is at 16MHz
    /// prior to calling this function.
    pub fn init(
        ccm: &mut ral::ccm::CCM,
        flexio: flexio::Instance<N>,
        pins: PINS,
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

        //////////// Configure FlexIO registers /////////////////
        let mut driver = DriverBuilder::new(flexio, pins);

        const DATA_SHIFTER: u8 = 0;
        const SHIFTER_TIMER: u8 = 0;
        const LOW_BIT_TIMER: u8 = 1;
        const HIGH_BIT_TIMER: u8 = 2;
        const IDLE_TIMER: u8 = 3;

        let shift_output_pin = PINS::FLEXIO_PIN_OFFSETS[0];
        let shift_timer_output_pin = PINS::FLEXIO_PIN_OFFSETS[1];
        let neopixel_output_pin = PINS::FLEXIO_PIN_OFFSETS[2];
        let idle_timer_output_pin = Some(PINS::FLEXIO_PIN_OFFSETS[3]);

        driver.configure_shifter(DATA_SHIFTER, SHIFTER_TIMER, shift_output_pin);
        driver.configure_shift_timer(SHIFTER_TIMER, DATA_SHIFTER, shift_timer_output_pin);
        driver.configure_low_bit_timer(LOW_BIT_TIMER, shift_timer_output_pin, neopixel_output_pin);
        driver.configure_high_bit_timer(HIGH_BIT_TIMER, shift_output_pin, neopixel_output_pin);
        driver.configure_idle_timer(IDLE_TIMER, shift_timer_output_pin, idle_timer_output_pin);

        // for (pos, pin) in PINS::FLEXIO_PIN_OFFSETS.iter().copied().enumerate() {
        //     //ral::write_reg!(ral::flexio, flexio,)
        // }

        Ok(driver.build())
    }

    /// A dummy function for development purposes
    pub fn dummy_write(&mut self) {
        ral::write_reg!(ral::flexio, self.flexio, SHIFTBUFBIS[0], 0x555000ff);

        // Clear timer register
        ral::write_reg!(ral::flexio, self.flexio, TIMSTAT, 0b1000);

        // Write data
        while ral::read_reg!(ral::flexio, self.flexio, SHIFTSTAT) == 0 {}
        ral::write_reg!(ral::flexio, self.flexio, SHIFTBUFBIS[0], 0x555000ff);
        while ral::read_reg!(ral::flexio, self.flexio, SHIFTSTAT) == 0 {}
        ral::write_reg!(ral::flexio, self.flexio, SHIFTBUFBIS[0], 0x555000fe);
        while ral::read_reg!(ral::flexio, self.flexio, SHIFTSTAT) == 0 {}

        // Wait for transfer finished
        while (ral::read_reg!(ral::flexio, self.flexio, TIMSTAT) & 0b1000) == 0 {}
    }
}

// TODO: Add PreparedPixels struct that contains prepared pixels.
// TODO: Add Iterator<Item = (Pixel, Pixel)> based pixel-setter function.
