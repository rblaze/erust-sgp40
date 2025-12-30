use core::fmt;
use embedded_hal::i2c::I2c;

use crate::sensirion::*;

pub mod commands;

const ADDR: u8 = 0x62;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Variant {
    SCD40,
    SCD41,
    SCD43,
}

impl fmt::Display for Variant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct SCD4x<I2C> {
    sensor: Sensor<I2C>,
}

impl<I2C: I2c> SCD4x<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self {
            sensor: Sensor::new(i2c, ADDR),
        }
    }

    /// Polls the sensor for whether data from a periodic or single shot measurement is ready to be read out.
    /// Returns true if successful, false if failed.
    pub fn get_data_ready_status(&mut self) -> Result<bool, Error<I2C::Error>> {
        let status = self
            .sensor
            .one_word_command(&commands::GET_DATA_READY_STATUS)?;

        // From the datasheet, if the 11 LSB are 0, data is not ready.
        Ok((status & 0x7FF) != 0)
    }

    /// Reading out the serial number can be used to identify the chip and to verify the presence of the sensor.
    pub fn get_serial_number(&mut self) -> Result<u64, Error<I2C::Error>> {
        let words = self
            .sensor
            .three_words_command(&commands::GET_SERIAL_NUMBER)?;

        Ok((words[0] as u64) << 32 | (words[1] as u64) << 16 | (words[2] as u64))
    }

    /// The perform_self_test command can be used as an end-of-line test to check the sensor functionality.
    /// Returns true if no malfunction detected, false if failed.
    pub async fn perform_self_test<Waiter: embedded_hal_async::delay::DelayNs>(
        &mut self,
        waiter: &mut Waiter,
    ) -> Result<bool, Error<I2C::Error>> {
        self.start_self_test()?;
        waiter.delay_ms(10000).await;
        self.read_self_test_result()
    }

    pub fn start_self_test(&mut self) -> Result<(), Error<I2C::Error>> {
        self.sensor.send_command(&commands::PERFORM_SELF_TEST)?;
        Ok(())
    }

    pub fn read_self_test_result(&mut self) -> Result<bool, Error<I2C::Error>> {
        let status = self.sensor.read_response_word()?;

        Ok(status == 0)
    }

    /// Reads out the SCD4x sensor variant
    pub fn get_sensor_variant(&mut self) -> Result<Variant, Error<I2C::Error>> {
        let status = self
            .sensor
            .one_word_command(&commands::GET_SENSOR_VARIANT)?;

        match status >> 12 {
            0b0000 => Ok(Variant::SCD40),
            0b0001 => Ok(Variant::SCD41),
            0b0101 => Ok(Variant::SCD43),
            _ => Err(Error::InvalidResponse),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SCD4x;
    use embedded_hal::i2c::{Error, Operation};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum DummyError {
        InvalidTest,
    }

    impl Error for DummyError {
        fn kind(&self) -> embedded_hal::i2c::ErrorKind {
            match &self {
                DummyError::InvalidTest => embedded_hal::i2c::ErrorKind::Other,
            }
        }
    }

    struct DummyBus<'a> {
        pub response: &'a [u8],
    }

    impl embedded_hal::i2c::ErrorType for DummyBus<'_> {
        type Error = DummyError;
    }

    impl embedded_hal::i2c::I2c for DummyBus<'_> {
        fn transaction(
            &mut self,
            _address: u8,
            operations: &mut [embedded_hal::i2c::Operation],
        ) -> Result<(), Self::Error> {
            match operations {
                [Operation::Write(_), Operation::Read(response)] => {
                    if response.len() != self.response.len() {
                        return Err(DummyError::InvalidTest);
                    }

                    response.copy_from_slice(self.response);

                    Ok(())
                }
                [Operation::Read(response)] => {
                    if response.len() != self.response.len() {
                        return Err(DummyError::InvalidTest);
                    }

                    response.copy_from_slice(self.response);

                    Ok(())
                }
                // Other transactions are invalid
                _ => Err(DummyError::InvalidTest),
            }
        }
    }

    #[test]
    fn test_perform_self_test_success() {
        let bus = DummyBus {
            response: &[0x00, 0x00, 0x81],
        };
        let mut sensor = SCD4x::new(bus);

        assert_eq!(sensor.read_self_test_result(), Ok(true));
    }

    #[test]
    fn test_perform_self_test_fail() {
        let bus = DummyBus {
            response: &[0x14, 0x40, 0x51],
        };
        let mut sensor = SCD4x::new(bus);

        assert_eq!(sensor.read_self_test_result(), Ok(false));
    }

    #[test]
    fn test_get_serial_number() {
        let bus = DummyBus {
            response: &[0xf8, 0x96, 0x31, 0x9f, 0x07, 0xc2, 0x3b, 0xbe, 0x89],
        };
        let mut sensor = SCD4x::new(bus);

        assert_eq!(sensor.get_serial_number(), Ok(273325796834238));
    }

    #[test]
    fn test_get_data_ready_status_ready() {
        let bus = DummyBus {
            response: &[0x00, 0x01, 0xb0],
        };
        let mut sensor = SCD4x::new(bus);

        assert_eq!(sensor.get_data_ready_status(), Ok(true));
    }

    #[test]
    fn test_get_data_ready_status_not_ready() {
        let bus = DummyBus {
            response: &[0x80, 0x00, 0xa2],
        };
        let mut sensor = SCD4x::new(bus);

        assert_eq!(sensor.get_data_ready_status(), Ok(false));
    }

    #[test]
    fn test_get_sensor_variant_scd40() {
        let bus = DummyBus {
            response: &[0x04, 0x40, 0x3f],
        };
        let mut sensor = SCD4x::new(bus);
        assert!(matches!(
            sensor.get_sensor_variant(),
            Ok(super::Variant::SCD40)
        ));
    }

    #[test]
    fn test_get_sensor_variant_scd41() {
        let bus = DummyBus {
            response: &[0x14, 0x40, 0x51],
        };
        let mut sensor = SCD4x::new(bus);
        assert!(matches!(
            sensor.get_sensor_variant(),
            Ok(super::Variant::SCD41)
        ));
    }

    #[test]
    fn test_get_sensor_variant_scd43() {
        let bus = DummyBus {
            response: &[0x54, 0x41, 0xe9],
        };
        let mut sensor = SCD4x::new(bus);
        assert!(matches!(
            sensor.get_sensor_variant(),
            Ok(super::Variant::SCD43)
        ));
    }
}
