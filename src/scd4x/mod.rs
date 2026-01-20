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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Measurement {
    pub co2_ppm: u16,
    pub temp_celsius: f32,
    pub humidity_percent: f32,
    pub temp_raw: u16,
    pub humidity_raw: u16,
}

impl fmt::Display for Measurement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} ppm CO2, {:.1}°C, {:.1}% RH",
            self.co2_ppm, self.temp_celsius, self.humidity_percent
        )
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
            .execute_command_0a1r(&commands::GET_DATA_READY_STATUS)?;

        // From the datasheet, if the 11 LSB are 0, data is not ready.
        Ok((status & 0x7FF) != 0)
    }

    /// Reading out the serial number can be used to identify the chip and to verify the presence of the sensor.
    pub fn get_serial_number(&mut self) -> Result<u64, Error<I2C::Error>> {
        let words = self
            .sensor
            .execute_command_0a3r(&commands::GET_SERIAL_NUMBER)?;

        Ok((words[0] as u64) << 32 | (words[1] as u64) << 16 | (words[2] as u64))
    }

    /// The perform_self_test command can be used as an end-of-line test to check the sensor functionality.
    pub fn start_self_test(&mut self) -> Result<(), Error<I2C::Error>> {
        self.sensor.send_command(&commands::PERFORM_SELF_TEST)?;
        Ok(())
    }

    /// Returns true if no malfunction detected, false if failed.
    /// Result is available 10s after self-test is started.
    pub fn read_self_test_result(&mut self) -> Result<bool, Error<I2C::Error>> {
        let status = self.sensor.read_response_word()?;

        Ok(status == 0)
    }

    /// Reads out the SCD4x sensor variant
    pub fn get_sensor_variant(&mut self) -> Result<Variant, Error<I2C::Error>> {
        let status = self
            .sensor
            .execute_command_0a1r(&commands::GET_SENSOR_VARIANT)?;

        match status >> 12 {
            0b0000 => Ok(Variant::SCD40),
            0b0001 => Ok(Variant::SCD41),
            0b0101 => Ok(Variant::SCD43),
            _ => Err(Error::InvalidResponse),
        }
    }

    /// Command returns a sensor running in periodic measurement mode or low power
    /// periodic measurement mode back to the idle state, e.g. to then allow
    /// changing the sensor configuration or to save power.
    /// Note that the sensor will only respond to other commands 500 ms after the
    /// stop_periodic_measurement command has been issued.
    pub fn stop_periodic_measurement(&mut self) -> Result<(), Error<I2C::Error>> {
        self.sensor
            .send_command(&commands::STOP_PERIODIC_MEASUREMENTS)?;
        Ok(())
    }

    /// Starts the periodic measurement mode. The signal update interval is 5 seconds.
    pub fn start_periodic_measurement(&mut self) -> Result<(), Error<I2C::Error>> {
        self.sensor
            .send_command(&commands::START_PERIODIC_MEASUREMENTS)?;
        Ok(())
    }

    /// Starts the low power periodic measurement mode. The signal update
    /// interval is approximately 30 seconds.
    pub fn start_low_power_periodic_measurement(&mut self) -> Result<(), Error<I2C::Error>> {
        self.sensor
            .send_command(&commands::START_LOW_POWER_PERIODIC_MEASUREMENT)?;
        Ok(())
    }

    /// Reads the sensor output. The measurement data can only be read out once
    /// per signal update interval as the buffer is emptied upon read-out.
    /// If no data is available in the buffer, the sensor returns a NACK.
    /// To avoid a NACK response, the get_data_ready_status can be issued to
    /// check data status.
    pub fn read_measurement(&mut self) -> Result<Measurement, Error<I2C::Error>> {
        let response = self
            .sensor
            .execute_command_0a3r(&commands::READ_MEASUREMENT)?;
        Ok(Measurement {
            co2_ppm: response[0],
            temp_celsius: -45.0 + 175.0 * (response[1] as f32 / 65535.0),
            humidity_percent: 100.0 * response[2] as f32 / 65535.0,
            temp_raw: response[1],
            humidity_raw: response[2],
        })
    }

    pub fn persist_settings(&mut self) -> Result<(), Error<I2C::Error>> {
        self.sensor.send_command(&commands::PERSIST_SETTINGS)
    }

    pub fn perform_factory_reset(&mut self) -> Result<(), Error<I2C::Error>> {
        self.sensor.send_command(&commands::PERFORM_FACTORY_RESET)
    }

    pub fn reinit(&mut self) -> Result<(), Error<I2C::Error>> {
        self.sensor.send_command(&commands::REINIT)
    }

    pub fn measure_single_shot(&mut self) -> Result<(), Error<I2C::Error>> {
        self.sensor.send_command(&commands::MEASURE_SINGLE_SHOT)
    }

    pub fn measure_single_shot_rht_only(&mut self) -> Result<(), Error<I2C::Error>> {
        self.sensor
            .send_command(&commands::MEASURE_SINGLE_SHOT_RHT_ONLY)
    }

    pub fn power_down(&mut self) -> Result<(), Error<I2C::Error>> {
        self.sensor.send_command(&commands::POWER_DOWN)
    }

    pub fn wake_up(&mut self) -> Result<(), Error<I2C::Error>> {
        self.sensor.send_command(&commands::WAKE_UP)
    }

    pub fn set_temperature_offset(
        &mut self,
        t_offset_celsius: f32,
    ) -> Result<(), Error<I2C::Error>> {
        let arg = (t_offset_celsius * 65536.0 / 175.0) as u16;
        self.sensor
            .send_command_1a(&commands::SET_TEMPERATURE_OFFSET, arg)
    }

    pub fn get_temperature_offset(&mut self) -> Result<f32, Error<I2C::Error>> {
        let val = self
            .sensor
            .execute_command_0a1r(&commands::GET_TEMPERATURE_OFFSET)?;
        Ok(val as f32 * 175.0 / 65536.0)
    }

    pub fn set_sensor_altitude(&mut self, altitude_m: u16) -> Result<(), Error<I2C::Error>> {
        self.sensor
            .send_command_1a(&commands::SET_SENSOR_ALTITUDE, altitude_m)
    }

    pub fn get_sensor_altitude(&mut self) -> Result<u16, Error<I2C::Error>> {
        self.sensor
            .execute_command_0a1r(&commands::GET_SENSOR_ALTITUDE)
    }

    pub fn set_ambient_pressure(&mut self, pressure_hpa: u16) -> Result<(), Error<I2C::Error>> {
        self.sensor
            .send_command_1a(&commands::SET_AMBIENT_PRESSURE, pressure_hpa)
    }

    pub fn perform_forced_recalibration(
        &mut self,
        target_co2_ppm: u16,
    ) -> Result<i16, Error<I2C::Error>> {
        let val = self
            .sensor
            .execute_command_1a1r(&commands::PERFORM_FORCED_RECALIBRATION, target_co2_ppm)?;
        if val == 0xFFFF {
            Err(Error::CommandFailed) // FRC failed
        } else {
            Ok(((val as i32) - 0x8000) as i16)
        }
    }

    pub fn set_automatic_self_calibration_enabled(
        &mut self,
        enabled: bool,
    ) -> Result<(), Error<I2C::Error>> {
        let arg = if enabled { 1 } else { 0 };
        self.sensor
            .send_command_1a(&commands::SET_AUTOMATIC_SELF_CALIBRATION_ENABLED, arg)
    }

    pub fn get_automatic_self_calibration_enabled(&mut self) -> Result<bool, Error<I2C::Error>> {
        let val = self
            .sensor
            .execute_command_0a1r(&commands::GET_AUTOMATIC_SELF_CALIBRATION_ENABLED)?;
        Ok(val != 0)
    }

    pub fn set_automatic_self_calibration_target(
        &mut self,
        target_ppm: u16,
    ) -> Result<(), Error<I2C::Error>> {
        self.sensor
            .send_command_1a(&commands::SET_AUTOMATIC_SELF_CALIBRATION_TARGET, target_ppm)
    }

    pub fn get_automatic_self_calibration_target(&mut self) -> Result<u16, Error<I2C::Error>> {
        self.sensor
            .execute_command_0a1r(&commands::GET_AUTOMATIC_SELF_CALIBRATION_TARGET)
    }

    pub fn set_automatic_self_calibration_initial_period(
        &mut self,
        hours: u16,
    ) -> Result<(), Error<I2C::Error>> {
        self.sensor.send_command_1a(
            &commands::SET_AUTOMATIC_SELF_CALIBRATION_INITIAL_PERIOD,
            hours,
        )
    }

    pub fn get_automatic_self_calibration_initial_period(
        &mut self,
    ) -> Result<u16, Error<I2C::Error>> {
        self.sensor
            .execute_command_0a1r(&commands::GET_AUTOMATIC_SELF_CALIBRATION_INITIAL_PERIOD)
    }

    pub fn set_automatic_self_calibration_standard_period(
        &mut self,
        hours: u16,
    ) -> Result<(), Error<I2C::Error>> {
        self.sensor.send_command_1a(
            &commands::SET_AUTOMATIC_SELF_CALIBRATION_STANDARD_PERIOD,
            hours,
        )
    }

    pub fn get_automatic_self_calibration_standard_period(
        &mut self,
    ) -> Result<u16, Error<I2C::Error>> {
        self.sensor
            .execute_command_0a1r(&commands::GET_AUTOMATIC_SELF_CALIBRATION_STANDARD_PERIOD)
    }
}

#[cfg(test)]
mod tests {
    use super::SCD4x;
    use crate::debug_utils::DummyBus;

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

    #[test]
    fn test_get_measurement() {
        let bus = DummyBus {
            response: &[0x01, 0xf4, 0x33, 0x66, 0x67, 0xa2, 0x5e, 0xb9, 0x3c],
        };
        let mut sensor = SCD4x::new(bus);
        let result = sensor.read_measurement();
        println!("result: {:?}", result);
        assert!(matches!(
            result,
            Ok(m) if m.co2_ppm == 500
                && (m.temp_celsius * 100.0).floor() == 2500.0
                && (m.humidity_percent * 100.0).floor() == 3700.0
        ));
    }

    #[test]
    fn test_get_temperature_offset() {
        let bus = DummyBus {
            response: &[0x09, 0x12, 0x63],
        };
        let mut sensor = SCD4x::new(bus);
        assert_eq!(
            sensor
                .get_temperature_offset()
                .map(|v| (v * 100.0).floor() / 100.0),
            Ok(6.2)
        );
    }

    #[test]
    fn test_perform_forced_recalibration_success() {
        let bus = DummyBus {
            response: &[0x7f, 0xce, 0x7b],
        };
        let mut sensor = SCD4x::new(bus);
        assert_eq!(sensor.perform_forced_recalibration(400), Ok(-50));
    }

    #[test]
    fn test_perform_forced_recalibration_fail() {
        let bus = DummyBus {
            response: &[0xff, 0xff, 0xac],
        };
        let mut sensor = SCD4x::new(bus);
        assert!(matches!(
            sensor.perform_forced_recalibration(400),
            Err(crate::sensirion::Error::CommandFailed)
        ));
    }

    #[test]
    fn test_get_sensor_altitude() {
        let bus = DummyBus {
            response: &[0x04, 0x4c, 0x42],
        };
        let mut sensor = SCD4x::new(bus);
        assert_eq!(sensor.get_sensor_altitude(), Ok(1100));
    }
}
