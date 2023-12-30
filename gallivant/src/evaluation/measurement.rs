use std::ops::RangeInclusive;

use crate::error::Error;

////////////////////////////////////////////////////////////////
// types
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
// methods
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
    pub fn evaluate(mut self, measurement: &[u8]) -> Result<Option<Self>, Error> {
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
