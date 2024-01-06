use std::io::{self, Read, Write};

use crate::syntax::{Error, ParsedExpr};

use super::measurement::{self, Measurement, MeasurementTest};

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

/// Describes a two way transaction between a frontend and a device.
///
#[derive(Clone, Debug, PartialEq)]
pub struct Transaction {
    expression: ParsedExpr,
    txbytes: Vec<u8>,
    txcomplete: bool,
    device: Device,
    response: Vec<u8>,
    test: Option<MeasurementTest>,
}

////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub enum TransactionStatus {
    Success,
    Ongoing(Transaction),
}

////////////////////////////////////////////////////////////////

/// Device that a frontend may need to communcate with during script execution.
///
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Device {
    TCU,
    Printer,
}

////////////////////////////////////////////////////////////////

// #[derive(Debug)]
// pub enum ErrorReason {
//     TestFailure,
//     IOError(Box<dyn std::error::Error>),
// }

// ////////////////////////////////////////////////////////////////

// #[derive(Debug)]
// pub struct Error {
//     expression: Expr,
//     reason: ErrorReason,
//     detail: String,
// }

////////////////////////////////////////////////////////////////
// construction / conversion
////////////////////////////////////////////////////////////////

impl Transaction {
    pub fn with_tcu(
        expression: ParsedExpr,
        txbytes: Vec<u8>,
        test: Option<MeasurementTest>,
    ) -> Self {
        Self {
            expression,
            txbytes,
            txcomplete: false,
            device: Device::TCU,
            response: Vec::new(),
            test,
        }
    }

    pub fn with_printer(
        expression: ParsedExpr,
        txbytes: Vec<u8>,
        test: Option<MeasurementTest>,
    ) -> Self {
        Self {
            expression,
            txbytes,
            txcomplete: false,
            device: Device::Printer,
            response: Vec::new(),
            test,
        }
    }
}

// impl Error {
//     fn from_io_error(transaction: &Transaction, error: std::io::Error) -> Self {
//         Self {
//             expression: transaction.expression.clone(),
//             reason: ErrorReason::IOError(Box::new(error)),
//             detail: String::new(),
//         }
//     }

//     fn with_detail(mut self, detail: &str) -> Self {
//         self.detail = detail.to_owned();
//         self
//     }
// }

////////////////////////////////////////////////////////////////
// methods
////////////////////////////////////////////////////////////////

impl Transaction {
    pub fn bytes(&self) -> &[u8] {
        &self.txbytes
    }

    pub fn process<T: Read + Write>(mut self, port: &mut T) -> Result<TransactionStatus, Error> {
        // let into_ioerror = |error| Error::from_io_error(&self, error);

        // Send bytes if needed.
        if !self.txcomplete {
            // port.write_all(&self.txbytes).map_err(into_ioerror)?;

            port.write_all(&self.txbytes).expect("TCU write error");
            self.txcomplete = true;

            return if self.device == Device::Printer && self.test.is_none() {
                Ok(TransactionStatus::Success)
            } else {
                Ok(TransactionStatus::Ongoing(self))
            };
        }

        let response = {
            let mut buffer = Vec::new();

            loop {
                let mut byte = [0; 64];
                match port.read(&mut byte) {
                    Ok(0) => break,
                    Ok(count) => buffer.extend_from_slice(&byte[0..count]),

                    Err(error) => match error.kind() {
                        io::ErrorKind::TimedOut => break,
                        _ => panic!("{error}"),
                    },
                }
            }

            buffer
        };

        self.response.extend_from_slice(&response);
        self.evaluate_response()
    }

    fn evaluate_response(mut self) -> Result<TransactionStatus, Error> {
        // Find the number of expected \r characters.
        let echo_expected = self.device == Device::TCU;
        let expected_endings = if self.test.is_some() && echo_expected {
            2
        } else if self.test.is_some() || echo_expected {
            1
        } else {
            0
        };

        // No response expected.
        if expected_endings == 0 {
            return Ok(TransactionStatus::Success);
        }

        let parts: Vec<&[u8]> = self.response.split_inclusive(|&b| b == b'\r').collect();

        // Incomplete response.
        if parts.len() < expected_endings {
            return Ok(TransactionStatus::Ongoing(self));
        }

        let (echo, measurement) = if echo_expected {
            (parts.get(0), parts.get(1))
        } else {
            (None, parts.get(0))
        };

        // Validate the echo.
        if echo.is_some_and(|echo| *echo != self.txbytes) {
            todo!("Command echo incorrect");
        }

        // Test the measurement.
        if let Some(test) = self.test {
            let measurement = *measurement.unwrap(); // Already checked that the measurement exists.
            let measurement = Measurement::try_from(measurement)
                .unwrap_or_else(|_| todo!("Handle measurement parsing failure"));

            match test.test(measurement) {
                Ok(_) => (),
                Err(measurement::Error::TestFailedRetryable(test)) => {
                    self.test = Some(test);
                    self.txcomplete = false;
                    return Ok(TransactionStatus::Ongoing(self));
                }
                Err(measurement::Error::TestFailed(test)) => todo!(),
                _ => todo!(),
            }
        }

        // Success.
        Ok(TransactionStatus::Success)
    }
}

////////////////////////////////////////////////////////////////
