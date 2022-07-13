use std::fmt;

// https://www.matthewflickinger.com/lab/whatsinagif/scripts/data_helpers.js
pub struct BitReader {
    bytes: Vec<u8>,
    byteOffset: usize,
    bitOffset: usize,
    totalByteOffset: usize,
}
impl BitReader {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            byteOffset: 0,
            bitOffset: 0,
            totalByteOffset: 0,
        }
    }
    pub fn read_bits(&mut self, len: usize) -> Result<u8, ByteSizeError> {
        let mut result = 0;
        let mut rbits: usize = 0;
        while rbits < len {
            if self.byteOffset >= self.bytes.len() {
                return Err(ByteSizeError {
                    len,
                    rbits: Some(rbits),
                    file: file!(),
                    line: line!(),
                    column: column!(),
                });
            }
            let bbits = std::cmp::min(8 - self.bitOffset, len - rbits);
            let mask = (0xFF >> (8 - bbits)) << self.bitOffset;
            result += ((self.bytes[self.byteOffset] & mask) >> self.bitOffset) << rbits;
            rbits += bbits;
            self.bitOffset += bbits;
            if (self.bitOffset == 8) {
                self.byteOffset += 1;
                self.totalByteOffset += 1;
                self.bitOffset = 0;
            }
        }
        Ok(result)
    }

    pub fn has_bits(&mut self, len: usize) -> Result<bool, ByteSizeError> {
        if (len > 12) {
            return Err(ByteSizeError {
                len,
                rbits: None,
                file: file!(),
                line: line!(),
                column: column!(),
            });
        }
        if self.byteOffset >= self.bytes.len() {
            return Ok(false);
        }
        let bitsRemain = 8 - self.bitOffset;
        if len <= bitsRemain {
            return Ok(true);
        }
        let bytesRemain = self.bytes.len() - self.byteOffset - 1;
        if bytesRemain < 1 {
            return Ok(false);
        }
        if len > bitsRemain + 8 * bytesRemain {
            return Ok(false);
        }
        return Ok(true);
    }

    pub fn set_bytes(&mut self, bytes: Vec<u8>, byteOffset: usize, bitOffset: usize) {
        self.bytes = bytes;
        self.byteOffset = byteOffset;
        self.bitOffset = bitOffset;
    }

    pub fn push_byte(&mut self, byte: u8) -> Option<ByteSizeError> {
        match self.has_bits(0) {
            Ok(has_bits) => {
                if (has_bits) {
                    self.bytes.push(byte);
                    self.byteOffset = 0;
                } else {
                    self.bytes.push(byte);
                    self.byteOffset = 0;
                    self.bitOffset = 0;
                }
                None
            }
            Err(err) => Some(err),
        }
    }

    pub fn get_state(&mut self) -> BitState {
        BitState {
            bitOffset: self.bitOffset,
            byteOffset: self.totalByteOffset,
        }
    }
}

pub struct ByteSizeError {
    len: usize,
    rbits: Option<usize>,
    file: &'static str,
    line: u32,
    column: u32,
}

impl fmt::Display for ByteSizeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_msg = match self.rbits {
            Some(rbits) => format!(
                "Not enough bytes to read {} bits (read {} bits) --> {}:{}:{}",
                self.len,
                rbits,
                self.file,
                self.line,
                self.column
            ),
            None => format!(
                "Exceeds max bit size: ${} (max: 12) --> {}:{}:{}",
                self.len,
                self.file,
                self.line,
                self.column
            ),
        };
        write!(f, "{}", err_msg)
    }
}

pub struct BitState {
    bitOffset: usize,
    byteOffset: usize,
}