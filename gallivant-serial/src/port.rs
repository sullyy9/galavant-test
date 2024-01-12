use std::time::Duration;

use serialport::{self, Error, ErrorKind, SerialPort, SerialPortBuilder};

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

/// Wrapper around SerialPort to allow opening and closing a port on the fly.
///
#[derive(Debug)]
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
// traits
////////////////////////////////////////////////////////////////

impl std::io::Read for CommPort {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            CommPort::Closed(_) => Err(std::io::Error::other(Error::new(
                ErrorKind::NoDevice,
                "Port closed",
            ))),
            CommPort::Open(port) => port.read(buf),
        }
    }
}

////////////////////////////////////////////////////////////////

impl std::io::Write for CommPort {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed").into()),
            CommPort::Open(port) => port.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed").into()),
            CommPort::Open(port) => port.flush(),
        }
    }
}

////////////////////////////////////////////////////////////////

impl SerialPort for CommPort {
    fn name(&self) -> Option<String> {
        match self {
            CommPort::Closed(_) => None,
            CommPort::Open(port) => port.name(),
        }
    }

    fn baud_rate(&self) -> serialport::Result<u32> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.baud_rate(),
        }
    }

    fn data_bits(&self) -> serialport::Result<serialport::DataBits> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.data_bits(),
        }
    }

    fn flow_control(&self) -> serialport::Result<serialport::FlowControl> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.flow_control(),
        }
    }

    fn parity(&self) -> serialport::Result<serialport::Parity> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.parity(),
        }
    }

    fn stop_bits(&self) -> serialport::Result<serialport::StopBits> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.stop_bits(),
        }
    }

    fn timeout(&self) -> Duration {
        match self {
            CommPort::Closed(_) => Duration::from_secs(0),
            CommPort::Open(port) => port.timeout(),
        }
    }

    fn set_baud_rate(&mut self, baud_rate: u32) -> serialport::Result<()> {
        match self {
            CommPort::Closed(port) => {
                *port = port.clone().baud_rate(baud_rate);
                Ok(())
            }
            CommPort::Open(port) => port.set_baud_rate(baud_rate),
        }
    }

    fn set_data_bits(&mut self, data_bits: serialport::DataBits) -> serialport::Result<()> {
        match self {
            CommPort::Closed(port) => {
                *port = port.clone().data_bits(data_bits);
                Ok(())
            }
            CommPort::Open(port) => port.set_data_bits(data_bits),
        }
    }

    fn set_flow_control(
        &mut self,
        flow_control: serialport::FlowControl,
    ) -> serialport::Result<()> {
        match self {
            CommPort::Closed(port) => {
                *port = port.clone().flow_control(flow_control);
                Ok(())
            }
            CommPort::Open(port) => port.set_flow_control(flow_control),
        }
    }

    fn set_parity(&mut self, parity: serialport::Parity) -> serialport::Result<()> {
        match self {
            CommPort::Closed(port) => {
                *port = port.clone().parity(parity);
                Ok(())
            }
            CommPort::Open(port) => port.set_parity(parity),
        }
    }

    fn set_stop_bits(&mut self, stop_bits: serialport::StopBits) -> serialport::Result<()> {
        match self {
            CommPort::Closed(port) => {
                *port = port.clone().stop_bits(stop_bits);
                Ok(())
            }
            CommPort::Open(port) => port.set_stop_bits(stop_bits),
        }
    }

    fn set_timeout(&mut self, timeout: Duration) -> serialport::Result<()> {
        match self {
            CommPort::Closed(port) => {
                *port = port.clone().timeout(timeout);
                Ok(())
            }
            CommPort::Open(port) => port.set_timeout(timeout),
        }
    }

    fn write_request_to_send(&mut self, level: bool) -> serialport::Result<()> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.write_request_to_send(level),
        }
    }

    fn write_data_terminal_ready(&mut self, level: bool) -> serialport::Result<()> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.write_data_terminal_ready(level),
        }
    }

    fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.read_clear_to_send(),
        }
    }

    fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.read_data_set_ready(),
        }
    }

    fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.read_ring_indicator(),
        }
    }

    fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.read_carrier_detect(),
        }
    }

    fn bytes_to_read(&self) -> serialport::Result<u32> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.bytes_to_read(),
        }
    }

    fn bytes_to_write(&self) -> serialport::Result<u32> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.bytes_to_write(),
        }
    }

    fn clear(&self, buffer_to_clear: serialport::ClearBuffer) -> serialport::Result<()> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.clear(buffer_to_clear),
        }
    }

    fn try_clone(&self) -> serialport::Result<Box<dyn SerialPort>> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.try_clone(),
        }
    }

    fn set_break(&self) -> serialport::Result<()> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.set_break(),
        }
    }

    fn clear_break(&self) -> serialport::Result<()> {
        match self {
            CommPort::Closed(_) => Err(Error::new(ErrorKind::NoDevice, "Port closed")),
            CommPort::Open(port) => port.clear_break(),
        }
    }
}

////////////////////////////////////////////////////////////////
