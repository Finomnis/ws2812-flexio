use imxrt_hal as hal;
use imxrt_ral as ral;

use hal::ccm::clock_gate;
use ral::{flexio, Valid};

use super::{driver_builder::DriverBuilder, errors, pins::Pins, Ws2812Driver};
use crate::prepared_pixels::PreparedPixelsRef;

impl<const N: u8, const L: usize, PINS: Pins<N, L>> Ws2812Driver<N, L, PINS>
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

        if available_shifters < PINS::PIN_COUNT {
            return Err(errors::WS2812InitError::NotEnoughShifters);
        }

        // if available_timers < (PINS::PIN_COUNT * 4) {
        //     return Err(errors::WS2812InitError::NotEnoughTimers);
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

    fn get_shifter_id(&self, pin_pos: u8) -> u32 {
        pin_pos as u32
    }

    fn get_shifter_timer_id(&self, pin_pos: u8) -> u32 {
        4 * (pin_pos as u32) + 0
    }
    fn get_low_bit_timer_id(&self, pin_pos: u8) -> u32 {
        4 * (pin_pos as u32) + 1
    }
    fn get_high_bit_timer_id(&self, pin_pos: u8) -> u32 {
        4 * (pin_pos as u32) + 2
    }
    fn get_idle_timer_id(&self, pin_pos: u8) -> u32 {
        4 * (pin_pos as u32) + 3
    }

    fn shift_buffer_empty(&self, pin_pos: u8) -> bool {
        let mask = 1u32 << self.get_shifter_id(pin_pos);
        let result = (ral::read_reg!(ral::flexio, self.flexio, SHIFTSTAT) & mask) != 0;
        //log::info!("shift_buffer_empty({}) -> {:?}", pin_pos, result);
        result
    }

    fn fill_shift_buffer(&self, pin_pos: u8, data: u32) {
        //log::info!("fill_shift_buffer({}, {})", pin_pos, data);
        let buf_id = self.get_shifter_id(pin_pos) as usize;

        #[cfg(target_endian = "big")]
        ral::write_reg!(ral::flexio, self.flexio, SHIFTBUFBIS[buf_id], data);

        #[cfg(target_endian = "little")]
        ral::write_reg!(ral::flexio, self.flexio, SHIFTBUFBBS[buf_id], data);
    }

    fn reset_idle_timer_finished_flag(&mut self, pin_pos: u8) {
        //log::info!("reset_idle_timer_finished_flag({})", pin_pos);
        let mask = 1u32 << self.get_idle_timer_id(pin_pos);
        ral::write_reg!(ral::flexio, self.flexio, TIMSTAT, mask);
    }

    fn idle_timer_finished(&mut self, pin_pos: u8) -> bool {
        let mask = 1u32 << self.get_idle_timer_id(pin_pos);
        let result = (ral::read_reg!(ral::flexio, self.flexio, TIMSTAT) & mask) != 0;
        //log::info!("idle_timer_finished({}) -> {:?}", pin_pos, result);
        result
    }

    /// Writes pixels to an LED strip.
    ///
    /// The first data stream will be sent to the first pin in the pins tuple.
    ///
    /// If you only want to send data to some pins, set the other data streams to `None`.
    pub fn write(&mut self, data: [Option<&dyn PreparedPixelsRef>; L]) {
        let mut data_streams = data.map(|d| d.map(|d| d.get_dma_buffer()));

        // Wait for the buffer to idle and clear timer overflow flag
        for i in data_streams
            .iter()
            .enumerate()
            .filter_map(|(pos, d)| d.map(|_| pos))
        {
            let pin_pos = i.try_into().unwrap();

            while !self.shift_buffer_empty(pin_pos) {}
            self.reset_idle_timer_finished_flag(pin_pos);
        }

        // Write data, to all lanes simultaneously
        loop {
            let mut data_left = false;

            for (data_stream, pin_pos) in data_streams
                .iter_mut()
                .enumerate()
                .filter_map(|(p, d)| d.as_mut().map(|d| (d, p)))
            {
                let pin_pos = pin_pos.try_into().unwrap();

                if let Some((next_data_element, advanced_data_stream)) = data_stream.split_first() {
                    if self.shift_buffer_empty(pin_pos) {
                        self.fill_shift_buffer(pin_pos, *next_data_element);
                        *data_stream = advanced_data_stream;
                    }

                    if !data_stream.is_empty() {
                        data_left = true;
                    }
                }
            }

            if !data_left {
                break;
            }
        }

        // Wait for transfer finished
        for i in data_streams
            .iter()
            .enumerate()
            .filter_map(|(pos, d)| d.map(|_| pos))
        {
            let pin_pos = i.try_into().unwrap();

            while !self.shift_buffer_empty(pin_pos) {}
            while !self.idle_timer_finished(pin_pos) {}
        }
    }
}
