use std::ops::RangeInclusive;

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

/// A measurement returned from either the TCU or the printer's debug protocol.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Measurement(u32);

////////////////////////////////////////////////////////////////

/// A test to be performed on a measurement taken by a device.
///
#[derive(Clone, Debug, PartialEq)]
pub struct MeasurementTest {
    pub expected: RangeInclusive<u32>,
    pub retries: u32,
    pub failure_message: String,
}

////////////////////////////////////////////////////////////////

/// A test to be performed on a measurement taken by a device.
///
#[derive(Clone, Debug, PartialEq)]
pub struct FailedTest {
    pub measurement: u32,
    pub expected: RangeInclusive<u32>,
    pub message: String,
}

////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum Error {
    TestFailed(FailedTest),
    TestFailedRetryable(MeasurementTest),

    /// Parsing of a measurement failed.
    ParseError(Box<dyn std::error::Error>),
}

////////////////////////////////////////////////////////////////
// construction / conversion
////////////////////////////////////////////////////////////////

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Self {
        Self::ParseError(Box::new(error))
    }
}

////////////////////////////////////////////////////////////////

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Self::ParseError(Box::new(error))
    }
}

////////////////////////////////////////////////////////////////

impl TryFrom<&[u8]> for Measurement {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let measurement = std::str::from_utf8(bytes)?;
        let measurement = measurement
            .chars()
            .take_while(|&c| c != '\r')
            .collect::<String>();

        let measurement = u32::from_str_radix(&measurement, 16)?;
        Ok(Measurement(measurement))
    }
}

////////////////////////////////////////////////////////////////

impl FailedTest {
    fn from_test_and_measurement(test: MeasurementTest, measurement: Measurement) -> Self {
        let Measurement(measurement) = measurement;
        Self {
            measurement,
            expected: test.expected,
            message: test.failure_message,
        }
    }
}

////////////////////////////////////////////////////////////////
// methods
////////////////////////////////////////////////////////////////

impl MeasurementTest {
    /// Test a measurement.
    ///
    /// # Arguments
    ///
    /// * `measurement` - Measurement to test.
    ///
    /// # Returns
    /// Result where the Ok value indicates the test was successfull.
    ///
    pub fn test(mut self, Measurement(measurement): Measurement) -> Result<(), Error> {
        let test_success = self.expected.contains(&measurement);

        if !test_success {
            return if self.retries > 0 {
                self.retries -= 1;
                Err(Error::TestFailedRetryable(self))
            } else {
                Err(Error::TestFailed(FailedTest::from_test_and_measurement(
                    self,
                    Measurement(measurement),
                )))
            };
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////
// ...
////////////////////////////////////////////////////////////////

impl std::fmt::Display for Measurement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

////////////////////////////////////////////////////////////////

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TestFailed(test) => write!(
                f,
                "Test failed, expected between {} and {} but measured {}",
                test.expected.start(),
                test.expected.end(),
                test.measurement
            ),
            Error::TestFailedRetryable(test) => {
                write!(f, "Test failed, retries remaining: {}", test.retries)
            }
            Error::ParseError(error) => write!(f, "{error}"),
        }
    }
}

////////////////////////////////////////////////////////////////

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::TestFailed(_) => None,
            Error::TestFailedRetryable(_) => None,
            Error::ParseError(error) => Some(error.as_ref()),
        }
    }
}

////////////////////////////////////////////////////////////////
