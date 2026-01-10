use embedded_hal::i2c::{Error, Operation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DummyError {
    InvalidTest,
}

impl Error for DummyError {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        match &self {
            DummyError::InvalidTest => embedded_hal::i2c::ErrorKind::Other,
        }
    }
}

pub struct DummyBus<'a> {
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
            [Operation::Write(_)] => Ok(()),
            // Other transactions are invalid
            _ => Err(DummyError::InvalidTest),
        }
    }
}
