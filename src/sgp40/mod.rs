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

    /// Performs sensor self-test.
    /// Returns true if successful, false if failed.
    pub fn self_test(&mut self) -> Result<bool, Error<I2C::Error>> {
        let result = self.sensor.one_word_command(&commands::EXECUTE_SELF_TEST)?;

        match result >> 8 {
            0xd4 => Ok(true),
            0x4b => Ok(false),
            _ => Err(Error::InvalidResponse),
        }
    }

    /// Reading out the serial number can be used to identify the chip and to verify the presence of the sensor.
    pub fn get_serial_number(&mut self) -> Result<u64, Error<I2C::Error>> {
        let words = self
            .sensor
            .three_words_command(&commands::GET_SERIAL_NUMBER)?;

        Ok((words[0] as u64) << 32 | (words[1] as u64) << 16 | (words[2] as u64))
    }
}
