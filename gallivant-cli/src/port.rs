use std::time::Duration;

use serialport::{self, Error, SerialPort, SerialPortBuilder};

////////////////////////////////////////////////////////////////

pub enum CommPort {
    Closed(SerialPortBuilder),
    Open(Box<dyn SerialPort>),
}

////////////////////////////////////////////////////////////////
// construction / conversion
////////////////////////////////////////////////////////////////

impl CommPort {
    pub fn open(&mut self) -> Result<(), Error> {
        match self {
            Self::Closed(port) => {
                *self = Self::Open(port.clone().open()?);
                Ok(())
            }
            Self::Open(_) => Ok(()),
        }
    }

    pub fn close(&mut self) -> Result<(), Error> {
        match self {
            Self::Closed(_) => Ok(()),
            Self::Open(port) => {
                *self = Self::Closed(
                    serialport::new(
                        port.name().expect("Failed to get port name"),
                        port.baud_rate()?,
                    )
                    .timeout(Duration::from_millis(100)),
                );
                Ok(())
            }
        }
    }
}

////////////////////////////////////////////////////////////////

impl From<SerialPortBuilder> for CommPort {
    fn from(port: SerialPortBuilder) -> Self {
        Self::Closed(port)
    }
}

////////////////////////////////////////////////////////////////

impl From<Box<dyn SerialPort>> for CommPort {
    fn from(port: Box<dyn SerialPort>) -> Self {
        Self::Open(port)
    }
}

////////////////////////////////////////////////////////////////
