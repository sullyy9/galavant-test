use std::time::Duration;

use serialport::{
    self, DataBits, Error, ErrorKind, FlowControl, Parity, SerialPort, SerialPortBuilder, StopBits,
};

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

/// Re-implementation of serialport::SerialPortBuilder because we would like provide read access to
/// the fields please and thank you.
/// Unlike serialport::SerialPortBuilder, this doesn't allow changing the timeout period as we use
/// that internally to determine if the stream is empty or not when reading from it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommPortBuilder {
    path: String,
    baud_rate: u32,
    data_bits: DataBits,
    flow_control: FlowControl,
    parity: Parity,
    stop_bits: StopBits,
    timeout: Duration,
}

////////////////////////////////////////////////////////////////

/// Wrapper around SerialPort to allow opening and closing a port on the fly.
///
#[derive(Debug)]
pub enum CommPort {
    Closed(CommPortBuilder),
    Open(Box<dyn SerialPort>),
}

////////////////////////////////////////////////////////////////
// construction / conversion
////////////////////////////////////////////////////////////////

impl CommPort {
    pub fn builder<'a>(
        path: impl Into<std::borrow::Cow<'a, str>>,
        baud_rate: u32,
    ) -> CommPortBuilder {
        CommPortBuilder::new(path, baud_rate)
    }

    pub fn open(&mut self) -> Result<(), Error> {
        match self {
            Self::Closed(port) => {
                *self = Self::Open(SerialPortBuilder::from(port.clone()).open()?);
                Ok(())
            }
            Self::Open(_) => Ok(()),
        }
    }

    pub fn close(&mut self) -> Result<(), Error> {
        match self {
            Self::Closed(_) => Ok(()),
            Self::Open(port) => {
                *self = Self::Closed(CommPortBuilder::from_serial_port(port.as_ref())?);
                Ok(())
            }
        }
    }
}

////////////////////////////////////////////////////////////////

impl From<CommPortBuilder> for SerialPortBuilder {
    fn from(builder: CommPortBuilder) -> Self {
        serialport::new(builder.path, builder.baud_rate)
            .data_bits(builder.data_bits)
            .flow_control(builder.flow_control)
            .parity(builder.parity)
            .stop_bits(builder.stop_bits)
            .timeout(builder.timeout)
    }
}

////////////////////////////////////////////////////////////////

impl CommPortBuilder {
    pub fn new<'a>(path: impl Into<std::borrow::Cow<'a, str>>, baud_rate: u32) -> Self {
        CommPortBuilder {
            path: path.into().into_owned(),
            baud_rate,
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: Duration::from_millis(1),
        }
    }

    fn from_serial_port(port: &dyn SerialPort) -> Result<Self, serialport::Error> {
        let Some(port_name) = port.name() else {
            return Err(Error::new(
                ErrorKind::NoDevice,
                "Path cannot be recovered from serial port",
            ));
        };

        Ok(CommPortBuilder {
            path: port_name,
            baud_rate: port.baud_rate()?,
            data_bits: port.data_bits()?,
            flow_control: port.flow_control()?,
            parity: port.parity()?,
            stop_bits: port.stop_bits()?,
            timeout: port.timeout(),
        })
    }
}

////////////////////////////////////////////////////////////////

impl CommPortBuilder {
    /// Set the path to the serial port
    #[must_use]
    pub fn path<'a>(mut self, path: impl Into<std::borrow::Cow<'a, str>>) -> Self {
        self.path = path.into().as_ref().to_owned();
        self
    }

    /// Set the baud rate in symbols-per-second
    #[must_use]
    pub fn baud_rate(mut self, baud_rate: u32) -> Self {
        self.baud_rate = baud_rate;
        self
    }

    /// Set the number of bits used to represent a character sent on the line
    #[must_use]
    pub fn data_bits(mut self, data_bits: DataBits) -> Self {
        self.data_bits = data_bits;
        self
    }

    /// Set the type of signalling to use for controlling data transfer
    #[must_use]
    pub fn flow_control(mut self, flow_control: FlowControl) -> Self {
        self.flow_control = flow_control;
        self
    }

    /// Set the type of parity to use for error checking
    #[must_use]
    pub fn parity(mut self, parity: Parity) -> Self {
        self.parity = parity;
        self
    }

    /// Set the number of bits to use to signal the end of a character
    #[must_use]
    pub fn stop_bits(mut self, stop_bits: StopBits) -> Self {
        self.stop_bits = stop_bits;
        self
    }

    pub fn build(self) -> CommPort {
        CommPort::from(self)
    }
}

////////////////////////////////////////////////////////////////

impl<T: Into<CommPortBuilder>> From<T> for CommPort {
    fn from(port: T) -> Self {
        Self::Closed(port.into())
    }
}

////////////////////////////////////////////////////////////////ru

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
            CommPort::Open(port) => {
                // There's no guarantee that there'll be a termination character in the stream so
                // we need to read one byte at a time and use timeout to determine if the stream's
                // empty. Otherwise we'll either get a timeout error or get blocked here forever if
                // no timeout is set.
                let mut count = 0;
                for byte in buf {
                    let mut byte_read = [0; 1];
                    match port.read(&mut byte_read) {
                        Ok(_) => {
                            *byte = byte_read[0];
                            count += 1;
                        }
                        Err(error) => match error.kind() {
                            std::io::ErrorKind::TimedOut => break,
                            _ => return Err(error),
                        },
                    }
                }

                Ok(count)
            }
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
            CommPort::Closed(port) => Some(port.path.clone()),
            CommPort::Open(port) => port.name(),
        }
    }

    fn baud_rate(&self) -> serialport::Result<u32> {
        match self {
            CommPort::Closed(port) => Ok(port.baud_rate),
            CommPort::Open(port) => port.baud_rate(),
        }
    }

    fn data_bits(&self) -> serialport::Result<serialport::DataBits> {
        match self {
            CommPort::Closed(port) => Ok(port.data_bits),
            CommPort::Open(port) => port.data_bits(),
        }
    }

    fn flow_control(&self) -> serialport::Result<serialport::FlowControl> {
        match self {
            CommPort::Closed(port) => Ok(port.flow_control),
            CommPort::Open(port) => port.flow_control(),
        }
    }

    fn parity(&self) -> serialport::Result<serialport::Parity> {
        match self {
            CommPort::Closed(port) => Ok(port.parity),
            CommPort::Open(port) => port.parity(),
        }
    }

    fn stop_bits(&self) -> serialport::Result<serialport::StopBits> {
        match self {
            CommPort::Closed(port) => Ok(port.stop_bits),
            CommPort::Open(port) => port.stop_bits(),
        }
    }

    fn timeout(&self) -> Duration {
        match self {
            CommPort::Closed(port) => port.timeout,
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

    /// Changing timeout is not allowed >:(
    /// It's required to internally to make reading more erganomic.
    fn set_timeout(&mut self, _: Duration) -> serialport::Result<()> {
        Ok(())
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
