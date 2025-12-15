use embedded_hal::i2c::I2c;
use thiserror::Error;

pub type Cmd = [u8; 2];

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum Error<I2cError> {
    #[error("invalid response")]
    InvalidResponse,
    #[error("invalid CRC")]
    InvalidCrc,
    #[error(transparent)]
    I2c(#[from] I2cError),
}

impl<E> embedded_hal::i2c::Error for Error<E>
where
    E: embedded_hal::i2c::Error,
{
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        match self {
            Self::I2c(err) => err.kind(),
            _ => embedded_hal::i2c::ErrorKind::Other,
        }
    }
}

pub struct Sensor<I2C> {
    i2c: I2C,
    addr: u8,
}

impl<I2C> Sensor<I2C> {
    pub fn new(i2c: I2C, addr: u8) -> Self {
        Self { i2c, addr }
    }

    // https://sensirion.com/media/documents/296373BB/6203C5DF/Sensirion_Gas_Sensors_Datasheet_SGP40.pdf
    // Section 4.6
    fn crc(data: &[u8; 2]) -> u8 {
        let mut crc = 0xff;

        for byte in data {
            crc ^= byte;

            for _ in 0..8 {
                if crc & 0x80 != 0 {
                    crc = (crc << 1) ^ 0x31;
                } else {
                    crc <<= 1;
                }
            }
        }

        crc
    }

    fn check_crc<E>(data: &[u8; 3]) -> Result<(), Error<E>> {
        if Self::crc(&[data[0], data[1]]) != data[2] {
            Err(Error::InvalidCrc)
        } else {
            Ok(())
        }
    }
}

impl<I2C: I2c> Sensor<I2C> {
    pub fn read_word(&mut self, cmd: &Cmd) -> Result<u16, Error<I2C::Error>> {
        let mut result = [0u8; 3];

        self.i2c.write_read(self.addr, cmd, &mut result)?;
        Self::check_crc(&result)?;

        Ok(u16::from_be_bytes([result[0], result[1]]))
    }

    pub fn read_three_words(&mut self, cmd: &Cmd) -> Result<[u16; 3], Error<I2C::Error>> {
        let mut result = [0u8; 9];

        self.i2c.write_read(self.addr, cmd, &mut result)?;
        for piece in result.as_chunks::<3>().0 {
            Self::check_crc(piece)?;
        }

        Ok([
            u16::from_be_bytes([result[0], result[1]]),
            u16::from_be_bytes([result[3], result[4]]),
            u16::from_be_bytes([result[6], result[7]]),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::Sensor;
    use embedded_hal::i2c::Error;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum DummyError {}

    impl Error for DummyError {
        fn kind(&self) -> embedded_hal::i2c::ErrorKind {
            unimplemented!()
        }
    }

    #[test]
    fn test_crc() {
        assert_eq!(
            Sensor::<()>::check_crc::<DummyError>(&[0xbe, 0xef, 0x92]),
            Ok(())
        );
        assert_eq!(
            Sensor::<()>::check_crc::<DummyError>(&[0xbe, 0x01, 0x92]),
            Err(super::Error::InvalidCrc)
        );
    }
}
