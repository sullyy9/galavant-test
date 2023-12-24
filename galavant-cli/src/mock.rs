use std::{
    collections::VecDeque,
    io::{Read, Write},
};

use serialport::SerialPort;

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

pub struct MockTCUPort {
    write: VecDeque<u8>,
    read: VecDeque<u8>,
}

////////////////////////////////////////////////////////////////
// construction / conversion
////////////////////////////////////////////////////////////////

impl MockTCUPort {
    pub fn new() -> Self {
        Self {
            write: VecDeque::new(),
            read: VecDeque::new(),
        }
    }
}

////////////////////////////////////////////////////////////////

fn tcu_decode_byte(bytes: &[u8]) -> u8 {
    let bytes = std::str::from_utf8(bytes).unwrap();
    u8::from_str_radix(bytes, 16).unwrap()
}

fn tcu_format_measurement(measurement: u16) -> Vec<u8> {
    format!("{measurement:04X}\r").into_bytes()
}

////////////////////////////////////////////////////////////////

impl Write for MockTCUPort {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write.extend(buf);
        if let Some(pos) = self.write.iter().position(|b| *b == b'\r') {
            let command: Vec<u8> = self.write.drain(0..=pos).collect();

            self.read.extend(&command);

            match &command[..] {
                [b'P', ..] => (), // Print
                [b'C', ..] => (), // Close relay
                [b'O', ..] => (), // Open relay

                // Measure
                [b'M', channel @ .., b'\r'] => match tcu_decode_byte(channel) {
                    0x00 => self.read.extend(tcu_format_measurement(100)), // battery current
                    0x01 => self.read.extend(tcu_format_measurement(100)), // charger current
                    0x02 => self.read.extend(tcu_format_measurement(1)),   // cts level
                    0x03 => self.read.extend(tcu_format_measurement(5000)), // vbatt
                    0x04 => self.read.extend(tcu_format_measurement(100)), // vdrop 100ms
                    0x05 => self.read.extend(tcu_format_measurement(200)), // vdrop 500ms
                    0x06 => self.read.extend(tcu_format_measurement(300)), // vdrop 2s
                    0x07 => self.read.extend(tcu_format_measurement(400)), // vdrop 5s
                    0x08 => self.read.extend(tcu_format_measurement(41)),  // tcu software version
                    0x09 => self.read.extend(tcu_format_measurement(5000)), // vref
                    0x0A => self.read.extend(tcu_format_measurement(5000)), // battery voltage internal
                    0x0B => self.read.extend(tcu_format_measurement(5000)), // charger voltage internal
                    0x10 => self.read.extend(tcu_format_measurement(0)),    // BT address high
                    0x11 => self.read.extend(tcu_format_measurement(0)),    // BT address mid
                    0x12 => self.read.extend(tcu_format_measurement(0)),    // BT address low
                    0x13 => self.read.extend(tcu_format_measurement(1)),    // BT address ready
                    0x14 => self.read.extend(tcu_format_measurement(1)),    // BT connect pin=0
                    0x15 => self.read.extend(tcu_format_measurement(1)),    // BT connect pin=1234
                    0x16 => self.read.extend(tcu_format_measurement(1)),    // BT connect pin=4254
                    _ => panic!(
                        "Unhandled TCU measure channel: {:?}",
                        tcu_decode_byte(channel)
                    ),
                },

                // Measure from printer.
                [b'W', .., b1, b2, b'\r'] => match tcu_decode_byte(&[*b1, *b2]) {
                    0 => self.read.extend(tcu_format_measurement(5000)), // battery voltage
                    1 => self.read.extend(tcu_format_measurement(5000)), // charger voltage
                    2 => self.read.extend(tcu_format_measurement(5000)), // vcc voltage
                    3 => self.read.extend(tcu_format_measurement(5000)), // mech voltage
                    4 => self.read.extend(tcu_format_measurement(25)),   // head temp
                    5 => self.read.extend(tcu_format_measurement(0x40)), // paper sensor
                    6 => self.read.extend(tcu_format_measurement(1)),    // wake signal
                    7 => self.read.extend(tcu_format_measurement(0)),    // unused
                    8 => self.read.extend(tcu_format_measurement(0)),    // feed button
                    9 => self.read.extend(tcu_format_measurement(0)),    // unused
                    10 => self.read.extend(tcu_format_measurement(0)),   // rtc present
                    11 => self.read.extend(tcu_format_measurement(0)),   // unused
                    12 => self.read.extend(tcu_format_measurement(0)),   // checksum
                    13 => self.read.extend(tcu_format_measurement(0)),   // mech busy status
                    14 => self.read.extend(tcu_format_measurement(0)),   // unused
                    15 => self.read.extend(tcu_format_measurement(0)),   // unused
                    16 => self.read.extend(tcu_format_measurement(0)),   // BT address
                    17 => self.read.extend(tcu_format_measurement(0)),   // BT address
                    18 => self.read.extend(tcu_format_measurement(0)),   // BT address
                    19 => self.read.extend(tcu_format_measurement(0)),   // font lib present
                    32..=63 => self.read.extend(tcu_format_measurement(0)), // option
                    _ => panic!(
                        "Unhandled printer measure channel: {:?}",
                        tcu_decode_byte(&[*b1, *b2])
                    ),
                },
                _ => panic!("Unhandled TCU command {command:?}"),
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

////////////////////////////////////////////////////////////////

impl Read for MockTCUPort {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes = std::cmp::min(self.read.len(), buf.len());

        for b in buf.iter_mut().take(bytes) {
            *b = self.read.pop_front().unwrap();
        }

        Ok(bytes)
    }
}

////////////////////////////////////////////////////////////////

impl SerialPort for MockTCUPort {
    fn name(&self) -> Option<String> {
        todo!()
    }

    fn baud_rate(&self) -> serialport::Result<u32> {
        todo!()
    }

    fn data_bits(&self) -> serialport::Result<serialport::DataBits> {
        todo!()
    }

    fn flow_control(&self) -> serialport::Result<serialport::FlowControl> {
        todo!()
    }

    fn parity(&self) -> serialport::Result<serialport::Parity> {
        todo!()
    }

    fn stop_bits(&self) -> serialport::Result<serialport::StopBits> {
        todo!()
    }

    fn timeout(&self) -> std::time::Duration {
        todo!()
    }

    fn set_baud_rate(&mut self, _: u32) -> serialport::Result<()> {
        todo!()
    }

    fn set_data_bits(&mut self, _: serialport::DataBits) -> serialport::Result<()> {
        todo!()
    }

    fn set_flow_control(&mut self, _: serialport::FlowControl) -> serialport::Result<()> {
        todo!()
    }

    fn set_parity(&mut self, _: serialport::Parity) -> serialport::Result<()> {
        todo!()
    }

    fn set_stop_bits(&mut self, _: serialport::StopBits) -> serialport::Result<()> {
        todo!()
    }

    fn set_timeout(&mut self, _: std::time::Duration) -> serialport::Result<()> {
        todo!()
    }

    fn write_request_to_send(&mut self, _: bool) -> serialport::Result<()> {
        todo!()
    }

    fn write_data_terminal_ready(&mut self, _: bool) -> serialport::Result<()> {
        todo!()
    }

    fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
        todo!()
    }

    fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
        todo!()
    }

    fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
        todo!()
    }

    fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
        todo!()
    }

    fn bytes_to_read(&self) -> serialport::Result<u32> {
        Ok(self.read.len() as u32)
    }

    fn bytes_to_write(&self) -> serialport::Result<u32> {
        todo!()
    }

    fn clear(&self, _: serialport::ClearBuffer) -> serialport::Result<()> {
        todo!()
    }

    fn try_clone(&self) -> serialport::Result<Box<dyn SerialPort>> {
        todo!()
    }

    fn set_break(&self) -> serialport::Result<()> {
        todo!()
    }

    fn clear_break(&self) -> serialport::Result<()> {
        todo!()
    }
}

////////////////////////////////////////////////////////////////
