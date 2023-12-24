use std::{ops::RangeInclusive, time::Duration};

use chrono::{Datelike, Local, Timelike};

use crate::{
    error::Error,
    expression::{Expr, ExprKind},
};

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

// Will need to handle requests that will receive a response. e.g. For the TCUTEST command.
// Maybe pass a callback fn to call with the requests response?
#[derive(PartialEq, Clone, Debug)]
pub enum FrontendRequest {
    None,
    Wait(Duration),

    GuiPrint(String),
    GuiDialogue { kind: Dialog, message: String },

    TCUTransact(Transaction),
    TCUAwaitResponse(Transaction),
    TCUFlush,

    // Requests for direct communication with the printer i.e. not via the TCU.
    PrinterOpen,
    PrinterClose,
    PrinterTransmit(Vec<u8>),
    PrinterTransact(Transaction),
    PrinterAwaitResponse(Transaction),
}

type Request = FrontendRequest;

////////////////////////////////////////////////////////////////

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Dialog {
    Notification { await_close: bool },
}

////////////////////////////////////////////////////////////////

#[allow(clippy::upper_case_acronyms)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Device {
    TCU,
    Printer,
}

////////////////////////////////////////////////////////////////

#[derive(PartialEq, Clone, Debug)]
pub struct Transaction {
    expression: Expr,
    bytes: Vec<u8>,
    device: Device,
    response: Vec<u8>,
    echo: bool,
    test: Option<MeasurementTest>,
}

////////////////////////////////////////////////////////////////

#[derive(PartialEq, Clone, Debug)]
pub struct MeasurementTest {
    pub expected: RangeInclusive<u32>,
    pub retries: u32,
    pub failure_message: String,
}

////////////////////////////////////////////////////////////////

impl Transaction {
    pub fn tcu(expression: Expr, bytes: Vec<u8>) -> Self {
        Self {
            expression,
            bytes,
            device: Device::TCU,
            response: Vec::new(),
            echo: true,
            test: None,
        }
    }

    pub fn tcu_with_test(expression: Expr, bytes: Vec<u8>, test: MeasurementTest) -> Self {
        Self {
            expression,
            bytes,
            device: Device::TCU,
            response: Vec::new(),
            echo: true,
            test: Some(test),
        }
    }

    pub fn printer_with_test(expression: Expr, bytes: Vec<u8>, test: MeasurementTest) -> Self {
        Self {
            expression,
            bytes,
            device: Device::Printer,
            response: Vec::new(),
            echo: false,
            test: Some(test),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn evaluate(mut self, response: &[u8]) -> Result<FrontendRequest, Error> {
        self.response.extend_from_slice(response);

        let endings = self.response.iter().filter(|&&b| b == b'\r').count();
        let expected_endings = if self.test.is_some() && self.echo {
            2
        } else if self.test.is_some() || self.echo {
            1
        } else {
            0
        };

        // No response expected.
        if expected_endings == 0 {
            return Ok(Request::None);
        }

        // Incomplete response.
        if endings < expected_endings {
            return match self.device {
                Device::TCU => Ok(Request::TCUAwaitResponse(self)),
                Device::Printer => Ok(Request::PrinterAwaitResponse(self)),
            };
        }

        let (echo, measurement) = if self.echo {
            self.response
                .iter()
                .position(|&b| b == b'\r')
                .map(|i| (&response[0..=i], &response[(i + 1)..]))
                .map_or((None, None), |(echo, meas)| (Some(echo), Some(meas)))
        } else {
            let measurement = self
                .response
                .iter()
                .position(|&b| b == b'\r')
                .map(|i| &response[0..=i]);

            (None, measurement)
        };

        // Command not echo'd by the TCU.
        if self.echo && echo.is_none() || echo.is_some_and(|echo| echo != self.bytes) {
            todo!();
        }

        // Test the measurement.
        if let Some(test) = self.test {
            if let Some(retry) = test.evaluate(measurement.unwrap())? {
                self.test = Some(retry);
                return match self.device {
                    Device::TCU => Ok(Request::TCUTransact(self)),
                    Device::Printer => Ok(Request::PrinterTransact(self)),
                };
            }
        }

        // Success.
        Ok(Request::None)
    }
}

////////////////////////////////////////////////////////////////

impl MeasurementTest {
    /// Evaluate a measurement.
    ///
    /// # Arguments
    ///
    /// * `measurement` - Byte slice containing the measurement, made up of hexadecimal digits. Only
    ///                   bytes up to the first \r are processed.
    ///
    /// # Returns
    /// Result containing one of:
    /// * None if the measurement passed the test
    /// * Self if the measurement failed the test but the test should be retried.
    /// * Error if the measurement failed the test.
    ///
    fn evaluate(mut self, measurement: &[u8]) -> Result<Option<Self>, Error> {
        let measurement = std::str::from_utf8(measurement).unwrap();
        let measurement = measurement
            .chars()
            .take_while(|&c| c != '\r')
            .collect::<String>();

        if let Ok(measurement) = u32::from_str_radix(&measurement, 16) {
            if self.expected.contains(&measurement) {
                Ok(None)
            } else if self.retries > 0 {
                self.retries -= 1;
                Ok(Some(self))
            } else {
                panic!(
                    "Test failure. Expected: {:?} | Got: {measurement}",
                    self.expected
                )
            }
        } else {
            panic!("Measurment parse error for {measurement}");
        }
    }
}

////////////////////////////////////////////////////////////////

/// Format a byte into a hex representation using ascii characters. Return those characters as
/// bytes.
///
fn tcu_format_byte(byte: u8) -> Vec<u8> {
    format!("{byte:02X}").into_bytes()
}

////////////////////////////////////////////////////////////////

pub fn evaluate(expr: &Expr) -> Result<FrontendRequest, Error> {
    match expr.kind() {
        ExprKind::String(_) => panic!("Orphaned String"),
        ExprKind::UInt(_) => panic!("Orphaned UInt"),

        ExprKind::ScriptComment(_) => Ok(Request::None),

        ExprKind::HPMode => Ok(Request::None),

        ExprKind::Comment(arg) => {
            if let ExprKind::String(str) = arg.kind() {
                return Ok(Request::GuiPrint(str.to_owned()));
            }

            panic!("Invalid COMMENT arg {:?}", arg);
        }

        ExprKind::Wait(arg) => {
            if let ExprKind::UInt(milliseconds) = arg.kind() {
                return Ok(Request::Wait(Duration::from_millis((*milliseconds).into())));
            }

            panic!("Invalid WAIT arg {:?}", arg);
        }

        ExprKind::OpenDialog(arg) => {
            if let ExprKind::String(message) = arg.kind() {
                let kind = Dialog::Notification { await_close: false };
                let message = message.to_owned();
                return Ok(Request::GuiDialogue { kind, message });
            }

            panic!("Invalid OPENDIALOG arg {:?}", arg);
        }

        ExprKind::WaitDialog(arg) => {
            if let ExprKind::String(message) = arg.kind() {
                let kind = Dialog::Notification { await_close: true };
                let message = message.to_owned();
                return Ok(Request::GuiDialogue { kind, message });
            }

            panic!("Invalid WAITDIALOG arg {:?}", arg);
        }

        ExprKind::Flush => Ok(Request::TCUFlush),
        ExprKind::Protocol => Ok(Request::None),

        ExprKind::Print(args) => {
            let mut arg_bytes = Vec::new();
            for arg in args {
                if let ExprKind::String(str) = arg.kind() {
                    arg_bytes.extend_from_slice(str.as_bytes());
                } else if let ExprKind::UInt(uint) = arg.kind() {
                    debug_assert!(*uint <= 255);
                    arg_bytes.push(*uint as u8);
                } else {
                    panic!("Invalid PRINT arg {arg:?}")
                }
            }

            // Each byte needs to be transformed into an ascii hex representation.
            let arg_bytes: Vec<u8> = arg_bytes.into_iter().flat_map(tcu_format_byte).collect();

            if arg_bytes.len() > 255 {
                todo!(
                    "Print command is limited to 255 args due to requirement to transmit the number
                    of args as a byte. Need to return an error here."
                );
            }

            let mut bytes = vec![b'P'];
            bytes.extend_from_slice(&tcu_format_byte(arg_bytes.len() as u8));
            bytes.extend_from_slice(&arg_bytes);
            bytes.push(b'\r');

            Ok(Request::TCUTransact(Transaction::tcu(
                expr.to_owned(),
                bytes,
            )))
        }

        ExprKind::SetTimeFormat(arg) => {
            if let ExprKind::UInt(uint) = arg.kind() {
                let mut bytes = Vec::from("P051B007466".as_bytes());
                bytes.extend_from_slice(&tcu_format_byte(*uint as u8));
                bytes.push(b'\r');

                return Ok(Request::TCUTransact(Transaction::tcu(
                    expr.to_owned(),
                    bytes,
                )));
            }

            panic!()
        }

        ExprKind::SetTime => {
            let datetime = Local::now();
            let datetime = format!(
                "{:02}:{:02}:{:02},{:02}/{:02}/{:02}",
                datetime.hour(),
                datetime.minute(),
                datetime.second(),
                datetime.day(),
                datetime.month(),
                (datetime.year() - 1900) % 100
            );

            let datetime = datetime.into_bytes().into_iter().flat_map(tcu_format_byte);

            let mut bytes = Vec::from("P151B007473".as_bytes());
            bytes.extend(datetime);
            bytes.push(b'\r');

            Ok(Request::TCUTransact(Transaction::tcu(
                expr.to_owned(),
                bytes,
            )))
        }

        ExprKind::SetOption { option, setting } => {
            if let (ExprKind::UInt(option), ExprKind::UInt(setting)) =
                (option.kind(), setting.kind())
            {
                debug_assert!(*option <= 255);
                debug_assert!(*setting <= 255);
                let bytes = format!("P061B00004F{:02X}{:02X}\r", option, setting).into_bytes();
                return Ok(Request::TCUTransact(Transaction::tcu(
                    expr.to_owned(),
                    bytes,
                )));
            }

            panic!("Invalid SETOPTION args {option:?}, {setting:?}")
        }

        ExprKind::TCUClose(arg) => {
            if let ExprKind::UInt(relay) = arg.kind() {
                debug_assert!(*relay <= 255);
                return Ok(Request::TCUTransact(Transaction::tcu(
                    expr.to_owned(),
                    format!("C{:02X}\r", relay).into_bytes(),
                )));
            }

            panic!("Invalid TCUCLOSE arg {arg:?}")
        }

        ExprKind::TCUOpen(arg) => {
            if let ExprKind::UInt(relay) = arg.kind() {
                debug_assert!(*relay <= 255);
                return Ok(Request::TCUTransact(Transaction::tcu(
                    expr.to_owned(),
                    format!("O{:02X}\r", relay).into_bytes(),
                )));
            }

            panic!("Invalid TCUOPEN arg {arg:?}")
        }

        ExprKind::TCUTest {
            channel,
            min,
            max,
            retries,
            message,
        } => {
            let args = (
                channel.kind(),
                min.kind(),
                max.kind(),
                retries.kind(),
                message.kind(),
            );
            if let (
                ExprKind::UInt(channel),
                ExprKind::UInt(min),
                ExprKind::UInt(max),
                ExprKind::UInt(retries),
                ExprKind::String(message),
            ) = args
            {
                debug_assert!(*channel <= 255);

                return Ok(Request::TCUTransact(Transaction::tcu_with_test(
                    expr.clone(),
                    format!("M{channel:02X}\r").into_bytes(),
                    MeasurementTest {
                        expected: *min..=*max,
                        retries: *retries,
                        failure_message: message.to_owned(),
                    },
                )));
            }

            panic!("Invalid TCUTEST args {channel:?}, {min:?}, {max:?}, {retries:?}, {message:?}")
        }

        ExprKind::PrinterSet(arg) => {
            if let ExprKind::UInt(channel) = arg.kind() {
                debug_assert!(*channel <= 255);
                return Ok(Request::TCUTransact(Transaction::tcu(
                    expr.to_owned(),
                    format!("P051B000053{:02X}\r", channel).into_bytes(),
                )));
            }

            panic!("Invalid PRINTERSET arg {arg:?}")
        }

        ExprKind::PrinterTest {
            channel,
            min,
            max,
            retries,
            message,
        } => {
            let args = (
                channel.kind(),
                min.kind(),
                max.kind(),
                retries.kind(),
                message.kind(),
            );

            if let (
                ExprKind::UInt(channel),
                ExprKind::UInt(min),
                ExprKind::UInt(max),
                ExprKind::UInt(retries),
                ExprKind::String(message),
            ) = args
            {
                debug_assert!(*channel <= 255);

                return Ok(Request::TCUTransact(Transaction::tcu_with_test(
                    expr.clone(),
                    format!("W051B00004D{channel:02X}\r").into_bytes(),
                    MeasurementTest {
                        expected: *min..=*max,
                        retries: *retries,
                        failure_message: message.to_owned(),
                    },
                )));
            }

            panic!(
                "Invalid PRINTERTEST args {channel:?}, {min:?}, {max:?}, {retries:?}, {message:?}"
            )
        }

        ExprKind::IssueTest(_) => Ok(Request::None),
        ExprKind::TestResult { .. } => Ok(Request::None),

        ExprKind::USBOpen => Ok(Request::PrinterOpen),
        ExprKind::USBClose => Ok(Request::PrinterClose),

        ExprKind::USBPrint(args) => {
            let mut bytes = Vec::new();
            for arg in args {
                if let ExprKind::String(str) = arg.kind() {
                    bytes.extend_from_slice(str.as_bytes());
                } else if let ExprKind::UInt(uint) = arg.kind() {
                    debug_assert!(*uint <= 255);
                    bytes.push(*uint as u8);
                } else {
                    panic!("Invalid USBPRINT arg {arg:?}")
                }
            }

            Ok(Request::PrinterTransmit(bytes))
        }

        ExprKind::USBSetTimeFormat(arg) => {
            if let ExprKind::UInt(uint) = arg.kind() {
                let bytes = vec![0x1B, 0x00, b't', b'f', *uint as u8];
                return Ok(Request::PrinterTransmit(bytes));
            }

            panic!()
        }

        ExprKind::USBSetTime => {
            let datetime = Local::now();
            let datetime = format!(
                "{:02}:{:02}:{:02},{:02}/{:02}/{:02}",
                datetime.hour(),
                datetime.minute(),
                datetime.second(),
                datetime.day(),
                datetime.month(),
                (datetime.year() - 1900) % 100
            );

            let mut bytes = vec![0x1B, 0x00, b't', b's'];
            bytes.extend_from_slice(datetime.as_bytes());

            Ok(Request::PrinterTransmit(bytes))
        }

        ExprKind::USBSetOption { option, setting } => {
            if let (ExprKind::UInt(option), ExprKind::UInt(setting)) =
                (option.kind(), setting.kind())
            {
                debug_assert!(*option <= 255);
                debug_assert!(*setting <= 255);

                let bytes = vec![0x1B, 0x00, 0x00, b'O', *option as u8, *setting as u8];
                return Ok(Request::PrinterTransmit(bytes));
            }

            panic!("Invalid USBSETOPTION args {option:?}, {setting:?}")
        }

        ExprKind::USBPrinterSet(arg) => {
            if let ExprKind::UInt(channel) = arg.kind() {
                debug_assert!(*channel <= 255);
                return Ok(Request::PrinterTransmit(vec![
                    0x1B,
                    0x00,
                    0x00,
                    b'S',
                    *channel as u8,
                ]));
            }

            panic!("Invalid USBPRINTERSET arg {arg:?}")
        }

        ExprKind::USBPrinterTest {
            channel,
            min,
            max,
            retries,
            message,
        } => {
            let args = (
                channel.kind(),
                min.kind(),
                max.kind(),
                retries.kind(),
                message.kind(),
            );

            if let (
                ExprKind::UInt(channel),
                ExprKind::UInt(min),
                ExprKind::UInt(max),
                ExprKind::UInt(retries),
                ExprKind::String(message),
            ) = args
            {
                debug_assert!(*channel <= 255);

                return Ok(Request::PrinterTransact(Transaction::printer_with_test(
                    expr.clone(),
                    vec![0x1B, 0x00, 0x00, b'M', *channel as u8],
                    MeasurementTest {
                        expected: *min..=*max,
                        retries: *retries,
                        failure_message: message.to_owned(),
                    },
                )));
            }

            panic!(
                "Invalid USBPRINTERTEST args {channel:?}, {min:?}, {max:?}, {retries:?}, {message:?}"
            )
        }
    }
}

////////////////////////////////////////////////////////////////
