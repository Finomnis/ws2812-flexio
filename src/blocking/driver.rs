use imxrt_hal as hal;
use imxrt_ral as ral;

use hal::ccm::clock_gate;
use ral::{flexio, Valid};

use super::{interleaved_pixels::InterleavedPixels, Ws2812Driver};
use crate::{errors, flexio_configurator::FlexIOConfigurator, pixelstream::PixelStreamRef, Pins};

impl<const N: u8, const L: usize, PINS: Pins<N, L>> Ws2812Driver<N, L, PINS>
where
    flexio::Instance<N>: Valid,
{
    /// Initializes the FlexIO driver.
    ///
    /// IMPORTANT! Make sure that the clock input of the FlexIO instance is at 16MHz
    /// prior to calling this function.
    ///
    /// Note that not all amounts of pins are always supported.
    ///
    /// The limiting factor is most likely the number of FlexIO timers;
    /// we need 2 timers plus an additional two per pin.
    ///
    /// For example, if the FlexIO instance has 8 timers, it supports up to 3 pins.
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

        // Always u8 because our pins list is &[u8].
        let available_pins = available_pins as u8;

        log::debug!("Initializing FlexIO #{}.", N);
        log::debug!("    Version: {}.{}", version_major, version_minor);
        log::debug!("    Feature Set: {}", available_feature_set);
        log::debug!("    Peripherals:");
        log::debug!("        {} triggers", available_triggers);
        log::debug!("        {} pins", available_pins);
        log::debug!("        {} timers", available_timers);
        log::debug!("        {} shifters", available_shifters);
        log::debug!("Pin Offsets: {:?}", PINS::FLEXIO_PIN_OFFSETS);

        if available_shifters < 1 {
            return Err(errors::WS2812InitError::NotEnoughShifters);
        }

        if available_timers < 2 + u32::from(PINS::PIN_COUNT) * 2 {
            return Err(errors::WS2812InitError::NotEnoughTimers);
        }

        //////////// Configure FlexIO registers /////////////////
        let mut flexio = FlexIOConfigurator::new(flexio);

        // Find 4 consecutive pins for the shifter output
        let shifter_output_start_pin = {
            let mut start = 0;
            let mut found = false;
            for i in 0..available_pins as u8 {
                if PINS::FLEXIO_PIN_OFFSETS.contains(&i) {
                    start = i + 1;
                } else {
                    if i - start >= 3 {
                        // We found 4 consecutive free pins!
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                return Err(errors::WS2812InitError::NeedFourConsecutiveInternalPins);
            }
            start
        };

        // Find a free pin for the shift timer output
        let shift_timer_output_pin = {
            let mut found_pin = None;

            for i in 0..available_pins as u8 {
                if !PINS::FLEXIO_PIN_OFFSETS.contains(&i)
                    && !(shifter_output_start_pin..shifter_output_start_pin + 4).contains(&i)
                {
                    found_pin = Some(i);
                    break;
                }
            }

            if let Some(pin) = found_pin {
                pin
            } else {
                return Err(errors::WS2812InitError::NotEnoughPins);
            }
        };

        let data_shifter = Self::get_shifter_id();
        let shifter_timer = Self::get_shifter_timer_id();
        let idle_timer = Self::get_idle_timer_id();

        flexio.configure_shifter(data_shifter, shifter_timer, shifter_output_start_pin);
        flexio.configure_shift_timer(shifter_timer, data_shifter, shift_timer_output_pin);
        flexio.configure_idle_timer(idle_timer, shift_timer_output_pin, None);

        for (pin_pos, pin_id) in PINS::FLEXIO_PIN_OFFSETS.iter().copied().enumerate() {
            let pin_pos = pin_pos.try_into().unwrap();
            let low_bit_timer = Self::get_low_bit_timer_id(pin_pos);
            let high_bit_timer = Self::get_high_bit_timer_id(pin_pos);

            let neopixel_output_pin = pin_id;

            flexio.configure_low_bit_timer(
                low_bit_timer,
                shift_timer_output_pin,
                neopixel_output_pin,
            );
            flexio.configure_high_bit_timer(
                high_bit_timer,
                shifter_output_start_pin + pin_pos,
                neopixel_output_pin,
            );
        }

        // Configure pins and create driver object
        pins.configure();
        Ok(Self {
            _pins: pins,
            flexio: flexio.finish(),
        })
    }

    const fn get_shifter_id() -> u8 {
        0
    }

    const fn get_shifter_timer_id() -> u8 {
        0
    }
    const fn get_idle_timer_id() -> u8 {
        1
    }

    const fn get_low_bit_timer_id(pin_pos: u8) -> u8 {
        2 * pin_pos + 2
    }
    const fn get_high_bit_timer_id(pin_pos: u8) -> u8 {
        2 * pin_pos + 3
    }

    fn shift_buffer_empty(&self) -> bool {
        let mask = 1u32 << Self::get_shifter_id();
        (ral::read_reg!(ral::flexio, self.flexio, SHIFTSTAT) & mask) != 0
    }

    fn fill_shift_buffer(&self, data: u32) {
        let buf_id = usize::from(Self::get_shifter_id());
        ral::write_reg!(ral::flexio, self.flexio, SHIFTBUFBIS[buf_id], data);
    }

    fn reset_idle_timer_finished_flag(&mut self) {
        let mask = 1u32 << Self::get_idle_timer_id();
        ral::write_reg!(ral::flexio, self.flexio, TIMSTAT, mask);
    }

    fn idle_timer_finished(&mut self) -> bool {
        let mask = 1u32 << Self::get_idle_timer_id();
        (ral::read_reg!(ral::flexio, self.flexio, TIMSTAT) & mask) != 0
    }

    /// Writes pixels to an LED strip.
    ///
    /// If the strips are of different length, the shorter ones will be padded
    /// with `0` to match the longest strip.
    ///
    /// For technical reasons, an additional `[0, 0, 0]` pixel will be added at
    /// the of the transmission.
    pub fn write(&mut self, data: [&mut dyn PixelStreamRef; L]) {
        // Wait for the buffer to idle and clear timer overflow flag
        while !self.shift_buffer_empty() {}
        self.reset_idle_timer_finished_flag();

        // Write data
        for elem in InterleavedPixels::new(data) {
            self.fill_shift_buffer(elem);
            while !self.shift_buffer_empty() {}
        }

        // Wait for transfer finished
        while !self.idle_timer_finished() {}
    }
}
