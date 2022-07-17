use std::fmt;

// https://www.matthewflickinger.com/lab/whatsinagif/scripts/data_helpers.js
pub struct BitReader {
    bytes: Vec<u8>,
    byte_offset: usize,
    bit_offset: usize,
}
impl BitReader {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            byte_offset: 0,
            bit_offset: 0,
        }
    }
    fn shl_or(&mut self, val: u16, shift: usize, def: u16) -> u16 {
        [val << (shift & 15), def][((shift & !15) != 0) as usize]
    }
    fn shr_or(&mut self, val: u16, shift: usize, def: u16) -> u16 {
        [val >> (shift & 15), def][((shift & !15) != 0) as usize]
    }
    pub fn read_bits(&mut self, len: usize) -> Result<u16, ByteSizeError> {
        let mut result = 0;
        let mut rbits: usize = 0;
        while rbits < len {
            if self.byte_offset >= self.bytes.len() {
                return Err(ByteSizeError {
                    len, // Not enough bytes to read {len} bits
                    rbits: Some(rbits), // (read {rbits} bits) 
                    file: file!(),
                    line: line!(),
                    column: column!(),
                });
            }
            let bbits = std::cmp::min(8 - self.bit_offset, len - rbits);

            let temp = self.shr_or(0xFF, 8 - bbits, 0);
            let mask = self.shl_or(temp, self.bit_offset, 0);

            let temp = self.shr_or(self.bytes[self.byte_offset] as u16 & mask, self.bit_offset, 0);
            result += self.shl_or(temp, rbits, 0);
            rbits += bbits;
            self.bit_offset += bbits;
            if self.bit_offset == 8 {
                self.byte_offset += 1;
                self.bit_offset = 0;
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
        if self.byte_offset >= self.bytes.len() {
            return Ok(false);
        }
        let bitsRemain = 8 - self.bit_offset;
        if len <= bitsRemain {
            return Ok(true);
        }
        let bytesRemain = self.bytes.len() - self.byte_offset - 1;
        if bytesRemain < 1 {
            return Ok(false);
        }
        if len > bitsRemain + 8 * bytesRemain {
            return Ok(false);
        }
        return Ok(true);
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) -> Option<ByteSizeError> {
        match self.has_bits(1) {
            Ok(has_bits) => {
                if has_bits {
                    let mut new_bytes: Vec<u8> = self.bytes[self.byte_offset..self.bytes.len()].to_vec();
                    new_bytes.extend(bytes);
                    self.bytes = new_bytes;
                    self.byte_offset = 0;
                } else {
                    self.bytes = bytes.to_vec();
                    self.byte_offset = 0;
                    self.bit_offset = 0;
                }
                None
            }
            Err(err) => Some(err),
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