use imxrt_hal as hal;
use imxrt_ral as ral;

use hal::ccm::clock_gate;
use ral::{flexio, Valid};

use super::{driver_builder::DriverBuilder, errors, pins::Pins, Ws2812Driver};

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
