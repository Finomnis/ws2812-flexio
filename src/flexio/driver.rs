use imxrt_hal as hal;
use imxrt_ral as ral;

use hal::ccm::clock_gate;
use ral::{flexio, Valid};

use super::{dma::WS2812Dma, driver_builder::DriverBuilder, errors, pins::Pins, Ws2812Driver};
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

        if available_timers < (PINS::PIN_COUNT * 4) {
            return Err(errors::WS2812InitError::NotEnoughTimers);
        }

        //////////// Configure FlexIO registers /////////////////
        let mut driver = DriverBuilder::new(flexio, pins);

        let mut get_next_free_pin = {
            let mut next_free_pin = 0;
            move || {
                while PINS::FLEXIO_PIN_OFFSETS.contains(&next_free_pin) {
                    next_free_pin += 1;
                }
                if u32::from(next_free_pin) >= available_pins {
                    Err(errors::WS2812InitError::NotEnoughPins)
                } else {
                    let result_pin = next_free_pin;
                    next_free_pin += 1;
                    Ok(result_pin)
                }
            }
        };

        for (pin_pos, pin_id) in PINS::FLEXIO_PIN_OFFSETS.iter().copied().enumerate() {
            let pin_pos = pin_pos.try_into().unwrap();
            let data_shifter = Self::get_shifter_id(pin_pos);
            let shifter_timer = Self::get_shifter_timer_id(pin_pos);
            let low_bit_timer = Self::get_low_bit_timer_id(pin_pos);
            let high_bit_timer = Self::get_high_bit_timer_id(pin_pos);
            let idle_timer = Self::get_idle_timer_id(pin_pos);

            let neopixel_output_pin = pin_id;

            let idle_timer_output_pin = None;

            let shift_output_pin = get_next_free_pin()?;
            let shift_timer_output_pin = get_next_free_pin()?;

            driver.configure_shifter(data_shifter, shifter_timer, shift_output_pin);
            driver.configure_shift_timer(shifter_timer, data_shifter, shift_timer_output_pin);
            driver.configure_low_bit_timer(
                low_bit_timer,
                shift_timer_output_pin,
                neopixel_output_pin,
            );
            driver.configure_high_bit_timer(high_bit_timer, shift_output_pin, neopixel_output_pin);
            driver.configure_idle_timer(idle_timer, shift_timer_output_pin, idle_timer_output_pin);
        }

        Ok(driver.build())
    }

    fn get_shifter_id(pin_pos: u8) -> u8 {
        pin_pos
    }

    fn get_shifter_timer_id(pin_pos: u8) -> u8 {
        4 * pin_pos + 0
    }
    fn get_low_bit_timer_id(pin_pos: u8) -> u8 {
        4 * pin_pos + 1
    }
    fn get_high_bit_timer_id(pin_pos: u8) -> u8 {
        4 * pin_pos + 2
    }
    fn get_idle_timer_id(pin_pos: u8) -> u8 {
        4 * pin_pos + 3
    }

    fn shift_buffer_empty(&self, pin_pos: u8) -> bool {
        let mask = 1u32 << Self::get_shifter_id(pin_pos);
        (ral::read_reg!(ral::flexio, self.flexio, SHIFTSTAT) & mask) != 0
    }

    fn fill_shift_buffer(&self, pin_pos: u8, data: u32) {
        let buf_id = usize::from(Self::get_shifter_id(pin_pos));

        #[cfg(target_endian = "big")]
        ral::write_reg!(ral::flexio, self.flexio, SHIFTBUFBIS[buf_id], data);

        #[cfg(target_endian = "little")]
        ral::write_reg!(ral::flexio, self.flexio, SHIFTBUFBBS[buf_id], data);
    }

    fn reset_idle_timer_finished_flag(&mut self, pin_pos: u8) {
        let mask = 1u32 << Self::get_idle_timer_id(pin_pos);
        ral::write_reg!(ral::flexio, self.flexio, TIMSTAT, mask);
    }

    fn idle_timer_finished(&mut self, pin_pos: u8) -> bool {
        let mask = 1u32 << Self::get_idle_timer_id(pin_pos);
        (ral::read_reg!(ral::flexio, self.flexio, TIMSTAT) & mask) != 0
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

    /// Writes pixels to an LED strip.
    ///
    /// The first data stream will be sent to the first pin in the pins tuple.
    ///
    /// If you only want to send data to some pins, set the other data streams to `None`.
    pub fn write_dma<R, F>(
        &mut self,
        data: [Option<(&dyn PreparedPixelsRef, &mut hal::dma::channel::Channel, u32)>; L],
        concurrent_action: F,
    ) -> R
    where
        F: FnOnce() -> R,
    {
        let mut data_streams =
            data.map(|d| d.map(|(data, dma, dma_signal)| (data.get_dma_buffer(), dma, dma_signal)));

        // Wait for the buffer to idle and clear timer overflow flag
        for i in data_streams
            .iter()
            .enumerate()
            .filter_map(|(pos, d)| d.as_ref().map(|_| pos))
        {
            let pin_pos = i.try_into().unwrap();

            while !self.shift_buffer_empty(pin_pos) {}
            self.reset_idle_timer_finished_flag(pin_pos);
        }

        let flexio_reg = core::cell::RefCell::new(&mut self.flexio);

        let mut destination = WS2812Dma::new(flexio_reg, 0, 1);

        let result = if let Some((data_ref, channel, dma_signal)) = &mut data_streams[0] {
            let write = core::pin::pin!(hal::dma::peripheral::write(
                channel,
                data_ref,
                &mut destination,
            ));

            let mut write = cassette::Cassette::new(write);

            let active = match write.poll_on() {
                Some(e) => {
                    e.unwrap();
                    false
                }
                None => true,
            };
            let result = concurrent_action();
            if active {
                write.block_on().unwrap();
            }

            result
        } else {
            panic!();
        };

        // Wait for transfer finished
        for i in data_streams
            .iter()
            .enumerate()
            .filter_map(|(pos, d)| d.as_ref().map(|_| pos))
        {
            let pin_pos = i.try_into().unwrap();

            while !self.shift_buffer_empty(pin_pos) {}
            while !self.idle_timer_finished(pin_pos) {}
        }

        result
    }
}
