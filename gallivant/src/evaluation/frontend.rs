use std::time::Duration;

use crate::{error::Error, expression::Expr};

use super::measurement::MeasurementTest;

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

/// Requests for actions a frontend needs to perform during script execution.
///
#[derive(Clone, Debug, PartialEq)]
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

////////////////////////////////////////////////////////////////

/// Types of dialog a frontend may need to create during script execution.
///
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dialog {
    Notification,

    /// Dialog that should display a message and allow the user to either continue or stop the test.
    ManualInput,
}

////////////////////////////////////////////////////////////////

/// Device that a frontend may need to communcate with during script execution.
///
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Device {
    TCU,
    Printer,
}

////////////////////////////////////////////////////////////////

/// Describes a two way transaction between a frontend and a device.
///
#[derive(Clone, Debug, PartialEq)]
pub struct Transaction {
    expression: Expr,
    bytes: Vec<u8>,
    device: Device,
    response: Vec<u8>,
    test: Option<MeasurementTest>,
}

////////////////////////////////////////////////////////////////
// construction / conversion
////////////////////////////////////////////////////////////////

impl Transaction {
    pub fn with_tcu(expression: Expr, bytes: Vec<u8>, test: Option<MeasurementTest>) -> Self {
        Self {
            expression,
            bytes,
            device: Device::TCU,
            response: Vec::new(),
            test,
        }
    }

    pub fn with_printer(expression: Expr, bytes: Vec<u8>, test: Option<MeasurementTest>) -> Self {
        Self {
            expression,
            bytes,
            device: Device::Printer,
            response: Vec::new(),
            test,
        }
    }
}

////////////////////////////////////////////////////////////////
// methods
////////////////////////////////////////////////////////////////

impl Transaction {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn evaluate(mut self, response: &[u8]) -> Result<FrontendRequest, Error> {
        self.response.extend_from_slice(response);

        let endings = self.response.iter().filter(|&&b| b == b'\r').count();

        let echo_expected = matches!(self.device, Device::TCU);
        let expected_endings = if self.test.is_some() && echo_expected {
            2
        } else if self.test.is_some() || echo_expected {
            1
        } else {
            0
        };

        // No response expected.
        if expected_endings == 0 {
            return Ok(FrontendRequest::None);
        }

        // Incomplete response.
        if endings < expected_endings {
            return match self.device {
                Device::TCU => Ok(FrontendRequest::TCUAwaitResponse(self)),
                Device::Printer => Ok(FrontendRequest::PrinterAwaitResponse(self)),
            };
        }

        let (echo, measurement) = if echo_expected {
            self.response
                .iter()
                .position(|&b| b == b'\r')
                .map(|i| (&self.response[0..=i], &self.response[(i + 1)..]))
                .map_or((None, None), |(echo, meas)| (Some(echo), Some(meas)))
        } else {
            let measurement = self
                .response
                .iter()
                .position(|&b| b == b'\r')
                .map(|i| &self.response[0..=i]);

            (None, measurement)
        };

        // Command not echo'd by the TCU.
        if echo_expected && echo.is_none() || echo.is_some_and(|echo| echo != self.bytes) {
            todo!();
        }

        // Test the measurement.
        if let Some(test) = self.test {
            if let Some(retry) = test.evaluate(measurement.unwrap())? {
                self.test = Some(retry);
                return match self.device {
                    Device::TCU => Ok(FrontendRequest::TCUTransact(self)),
                    Device::Printer => Ok(FrontendRequest::PrinterTransact(self)),
                };
            }
        }

        // Success.
        Ok(FrontendRequest::None)
    }
}

////////////////////////////////////////////////////////////////
