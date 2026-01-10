use crate::sensirion::*;

pub mod commands;

use embedded_hal::i2c::I2c;

const ADDR: u8 = 0x59;

#[derive(Debug)]
pub struct SGP40<I2C> {
    sensor: Sensor<I2C>,
}

impl<I2C: I2c> SGP40<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self {
            sensor: Sensor::new(i2c, ADDR),
        }
    }

    /// This command triggers the built-in self-test checking
    /// for integrity of both hotplate and MOX material.
    /// Returns true if successful, false if failed.
    pub fn self_test(&mut self) -> Result<bool, Error<I2C::Error>> {
        let result = self.sensor.one_word_command(&commands::EXECUTE_SELF_TEST)?;

        match result >> 8 {
            0xd4 => Ok(true),
            0x4b => Ok(false),
            _ => Err(Error::InvalidResponse),
        }
    }

    /// Reading out the serial number can be used to identify the chip and
    /// to verify the presence of the sensor. Subsequently, the sensor enters idle mode.
    pub fn get_serial_number(&mut self) -> Result<u64, Error<I2C::Error>> {
        let words = self
            .sensor
            .three_words_command(&commands::GET_SERIAL_NUMBER)?;

        Ok((words[0] as u64) << 32 | (words[1] as u64) << 16 | (words[2] as u64))
    }

    /// This command turns the hotplate off and stops the measurement.
    pub fn turn_heater_off(&mut self) -> Result<(), Error<I2C::Error>> {
        self.sensor.send_command(&commands::TURN_HEATER_OFF)
    }

    /// Measure raw signal with humidity and temperature compensation.
    pub fn measure_raw_signal(
        &mut self,
        humidity_percent: f32,
        temp_celsius: f32,
    ) -> Result<u16, Error<I2C::Error>> {
        let rh_ticks = Self::rh_to_ticks(humidity_percent);
        let temp_ticks = Self::temp_to_ticks(temp_celsius);

        self.sensor
            .one_word_command_with_args(&commands::MEASURE_RAW_SIGNAL, rh_ticks, temp_ticks)
    }

    fn rh_to_ticks(rh: f32) -> u16 {
        (rh * 65535.0 / 100.0) as u16
    }

    fn temp_to_ticks(temp: f32) -> u16 {
        ((temp + 45.0) * 65535.0 / 175.0) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::SGP40;
    use crate::debug_utils::DummyBus;

    #[test]
    fn test_self_test_success() {
        let bus = DummyBus {
            response: &[0xd4, 0x00, 0xc6],
        };
        let mut sensor = SGP40::new(bus);

        assert_eq!(sensor.self_test(), Ok(true));
    }

    #[test]
    fn test_self_test_fail() {
        let bus = DummyBus {
            response: &[0x4b, 0x00, 0x12],
        };
        let mut sensor = SGP40::new(bus);

        assert_eq!(sensor.self_test(), Ok(false));
    }

    #[test]
    fn test_get_serial_number() {
        // Example serial from datasheet or constructed.
        // Word 0: 0x0000, CRC 0x81
        // Word 1: 0x0001, CRC 0xB0
        // Word 2: 0x0002, CRC 0xE3
        // Result: 0x000000010002
        let bus = DummyBus {
            response: &[0x00, 0x00, 0x81, 0x00, 0x01, 0xb0, 0x00, 0x02, 0xe3],
        };
        let mut sensor = SGP40::new(bus);

        assert_eq!(sensor.get_serial_number(), Ok(0x000000010002));
    }

    #[test]
    fn test_measure_raw_signal() {
        let bus = DummyBus {
            response: &[0xbe, 0xef, 0x92],
        };
        let mut sensor = SGP40::new(bus);

        let result = sensor.measure_raw_signal(50.0, 25.0);

        assert_eq!(result, Ok(0xbeef));
    }

    #[test]
    fn test_rh_to_ticks() {
        assert_eq!(SGP40::<DummyBus>::rh_to_ticks(0.0), 0);
        assert_eq!(SGP40::<DummyBus>::rh_to_ticks(50.0), 32767);
        assert_eq!(SGP40::<DummyBus>::rh_to_ticks(100.0), 65535);
    }

    #[test]
    fn test_temp_to_ticks() {
        assert_eq!(SGP40::<DummyBus>::temp_to_ticks(-45.0), 0);
        assert_eq!(SGP40::<DummyBus>::temp_to_ticks(25.0), 26214);
        assert_eq!(SGP40::<DummyBus>::temp_to_ticks(130.0), 65535);
    }
}
