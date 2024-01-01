use std::{
    collections::VecDeque,
    io::{Read, Write},
};

////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PortMock {
    pub rxdata: VecDeque<u8>,
    pub txdata: VecDeque<u8>,
}

////////////////////////////////////////////////////////////////

impl PortMock {
    pub fn new() -> Self {
        Self {
            rxdata: VecDeque::new(),
            txdata: VecDeque::new(),
        }
    }
}

////////////////////////////////////////////////////////////////

impl Read for PortMock {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut count = 0;
        for byte in buf {
            if let Some(rxbyte) = self.rxdata.pop_front() {
                *byte = rxbyte;
                count += 1;
            } else {
                break;
            }
        }

        Ok(count)
    }
}

////////////////////////////////////////////////////////////////

impl Write for PortMock {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.txdata.extend(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

////////////////////////////////////////////////////////////////
