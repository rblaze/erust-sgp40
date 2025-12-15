use crate::sensirion::*;

pub mod commands;

use embedded_hal::i2c::I2c;

const ADDR: u8 = 0x59;

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
        let result = self.sensor.read_word(&commands::CMD_EXECUTE_SELF_TEST)?;

        match result >> 8 {
            0xd4 => Ok(true),
            0x4b => Ok(false),
            _ => Err(Error::InvalidResponse),
        }
    }
}
